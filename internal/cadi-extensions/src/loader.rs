//! Extension loading and management

use async_trait::async_trait;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use cadi_core::AtomicChunk;

use crate::manifest::{validate_manifest, ExtensionManifest};
use crate::traits::*;
use crate::types::*;

/// Loader for CADI extensions
pub struct ExtensionLoader {
    search_paths: Vec<PathBuf>,
    loaded_libraries: HashMap<ExtensionId, Library>,
    loaded_extensions: HashMap<ExtensionId, LoadedExtension>,
    context: ExtensionContext,
}

impl ExtensionLoader {
    /// Create a new extension loader
    pub fn new() -> Self {
        Self {
            search_paths: vec![
                "./extensions".into(),
                dirs::home_dir()
                    .map(|h| h.join(".cadi").join("extensions"))
                    .unwrap_or_else(|| "./extensions".into()),
                "/usr/local/lib/cadi/extensions".into(),
            ],
            loaded_libraries: HashMap::new(),
            loaded_extensions: HashMap::new(),
            context: ExtensionContext {
                config: HashMap::new(),
                registry: Box::new(LocalRegistry::new()), // Default registry
            },
        }
    }

    /// Add a search path for extensions
    pub fn with_search_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.search_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Set the extension context
    pub fn with_context(mut self, context: ExtensionContext) -> Self {
        self.context = context;
        self
    }

    /// Load all extensions from search paths
    pub async fn load_all(&mut self) -> Result<()> {
        let search_paths = self.search_paths.clone();

        for path in search_paths {
            if path.exists() {
                self.load_from_directory(&path).await?;
            }
        }

        Ok(())
    }

    /// Load extensions from a directory
    async fn load_from_directory(&mut self, dir: &Path) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| ExtensionError::Io(e))?;

        for entry in entries {
            let entry = entry.map_err(|e| ExtensionError::Io(e))?;
            let path = entry.path();

            if path.is_dir() {
                self.load_from_extension_directory(&path).await?;
            }
        }

        Ok(())
    }

    /// Load a single extension from its directory
    async fn load_from_extension_directory(&mut self, dir: &Path) -> Result<()> {
        let manifest_path = dir.join("extension.toml");

        if !manifest_path.exists() {
            return Ok(()); // Not an extension directory
        }

        let manifest = ExtensionManifest::from_file(&manifest_path)?;

        // Validate manifest
        validate_manifest(&manifest)?;

        // Load the library
        let lib_path = self.find_library_file(dir)?;
        let library = unsafe {
            Library::new(&lib_path)
                .map_err(|e| ExtensionError::LoadFailed(format!("Failed to load {}: {}", lib_path.display(), e)))?
        };

        // Get the extension constructor
        let constructor: Symbol<unsafe extern "C" fn() -> *mut dyn Extension> = unsafe {
            library.get(b"cadi_extension_create")
                .map_err(|e| ExtensionError::LoadFailed(format!("Failed to find constructor: {}", e)))?
        };

        // Create the extension instance
        let extension_ptr = unsafe { constructor() };
        if extension_ptr.is_null() {
            return Err(ExtensionError::LoadFailed("Constructor returned null".into()));
        }

        let mut extension = unsafe { Box::from_raw(extension_ptr) };
        let metadata = extension.metadata();

        // Initialize the extension
        extension.initialize(&self.context).await?;

        let loaded = LoadedExtension {
            metadata: metadata.clone(),
            status: ExtensionStatus::Active,
            instance: extension,
        };

        self.loaded_libraries.insert(metadata.id.clone(), library);
        self.loaded_extensions.insert(metadata.id.clone(), loaded);

        Ok(())
    }

    /// Find the library file in an extension directory
    fn find_library_file(&self, dir: &Path) -> Result<PathBuf> {
        let lib_name = if cfg!(target_os = "windows") {
            "extension.dll"
        } else if cfg!(target_os = "macos") {
            "libextension.dylib"
        } else {
            "libextension.so"
        };

        let lib_path = dir.join(lib_name);
        if lib_path.exists() {
            Ok(lib_path)
        } else {
            Err(ExtensionError::LoadFailed(format!(
                "Library file not found: {}",
                lib_path.display()
            )))
        }
    }

    /// Get a loaded extension by ID
    pub fn get_extension(&self, id: &ExtensionId) -> Option<&LoadedExtension> {
        self.loaded_extensions.get(id)
    }

    /// Get all loaded extensions
    pub fn get_all_extensions(&self) -> Vec<&LoadedExtension> {
        self.loaded_extensions.values().collect()
    }

    /// Get extensions by type
    pub fn get_extensions_by_type(&self, extension_type: ExtensionType) -> Vec<&LoadedExtension> {
        self.loaded_extensions
            .values()
            .filter(|ext| ext.metadata.extension_type == extension_type)
            .collect()
    }

    /// Unload an extension
    pub async fn unload_extension(&mut self, id: &ExtensionId) -> Result<()> {
        if let Some(mut loaded) = self.loaded_extensions.remove(id) {
            loaded.instance.shutdown().await?;
        }

        if let Some(library) = self.loaded_libraries.remove(id) {
            drop(library); // This will unload the library
        }

        Ok(())
    }

    /// Shutdown all extensions
    pub async fn shutdown(&mut self) -> Result<()> {
        let ids: Vec<_> = self.loaded_extensions.keys().cloned().collect();

        for id in ids {
            self.unload_extension(&id).await?;
        }

        Ok(())
    }
}

impl Default for ExtensionLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Default local registry implementation
struct LocalRegistry {
    chunks: Arc<RwLock<HashMap<String, AtomicChunk>>>,
}

impl LocalRegistry {
    fn new() -> Self {
        Self {
            chunks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl RegistryExtension for LocalRegistry {
    async fn store(&self, chunk: &AtomicChunk) -> Result<String> {
        let id = chunk.chunk_id.clone();
        self.chunks.write().await.insert(id.clone(), chunk.clone());
        Ok(id)
    }

    async fn retrieve(&self, id: &str) -> Result<Option<AtomicChunk>> {
        Ok(self.chunks.read().await.get(id).cloned())
    }

    async fn search(&self, _query: &SearchQuery) -> Result<Vec<AtomicChunk>> {
        // Simple implementation - return all chunks
        Ok(self.chunks.read().await.values().cloned().collect())
    }
}

#[async_trait]
impl Extension for LocalRegistry {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: ExtensionId::new(),
            name: "local-registry".into(),
            version: "1.0.0".into(),
            description: "Local filesystem registry".into(),
            author: "CADI Team".into(),
            homepage: None,
            repository: None,
            license: "MIT".into(),
            extension_type: ExtensionType::Registry,
        }
    }

    async fn initialize(&mut self, _context: &ExtensionContext) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}