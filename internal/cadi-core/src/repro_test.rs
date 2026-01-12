
    #[test]
    fn test_smart_chunker_c_parsing_gap() {
        let chunker = SmartChunker::default();
        let source = r#"
            #include <stdio.h>
            
            void my_function() {
                printf("Hello");
            }
        "#;
        
        // This relies on the standard SmartChunker logic
        let analysis = chunker.analyze_file(Path::new("test.c"), source);
        
        // Detailed check
        let has_import = analysis.entities.iter().any(|e| e.kind == EntityKind::Import && e.name == "stdio.h");
        let has_function = analysis.entities.iter().any(|e| e.kind == EntityKind::Function && e.name == "my_function");
        
        println!("Has Import: {}", has_import);
        println!("Has Function: {}", has_function);
        
        // I expect imports to work (because I added extract_import_entities)
        // I expect functions to FAIL (because SmartChunker has no C parsing logic)
        assert!(has_import, "Should find imports");
        assert!(has_function, "Should find C functions (this is expected to fail currently)");
    }
