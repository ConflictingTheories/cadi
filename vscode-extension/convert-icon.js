const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

async function convertIcon() {
    const svgPath = path.join(__dirname, 'resources', 'icons', 'cadi-icon.svg');
    const pngPath = path.join(__dirname, 'assets', 'cadi-icon.png');

    try {
        await sharp(svgPath)
            .resize(128, 128)
            .png()
            .toFile(pngPath);

        console.log('Icon converted successfully!');
    } catch (error) {
        console.error('Error converting icon:', error);
    }
}

convertIcon();