#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Extract stellar object labels from EVE Frontier localization data
 * 
 * This script:
 * 1. Loads the starmapcache.json to get all solar system, constellation, and region IDs
 * 2. Loads localization_fsd_main.json to find message IDs for systems, constellations, and regions
 * 3. Loads localization_fsd_en-us.json to get the actual names
 * 4. Creates mappings of ID to name for all three object types
 */

function loadJsonFile(filePath) {
    try {
        console.log(`Loading ${filePath}...`);
        const data = fs.readFileSync(filePath, 'utf8');
        return JSON.parse(data);
    } catch (error) {
        console.error(`Error loading ${filePath}:`, error.message);
        process.exit(1);
    }
}

function extractStellarLabels() {
    const dataDir = path.join(__dirname, '../data/json');
    
    // Load all required data files
    const starMapCache = loadJsonFile(path.join(dataDir, 'starmapcache.json'));
    const mainLocalization = loadJsonFile(path.join(dataDir, 'localization_fsd_main.json'));
    const enUsLocalization = loadJsonFile(path.join(dataDir, 'localization_fsd_en-us.json'));
    
    console.log('Extracting IDs from starmapcache...');
    
    // Extract all unique IDs from starmapcache
    const allSystemIds = new Set();
    const allConstellationIds = new Set();
    const allRegionIds = new Set();
    
    // Extract region IDs and their content
    if (starMapCache.regions) {
        Object.keys(starMapCache.regions).forEach(id => {
            allRegionIds.add(parseInt(id));
        });
        
        Object.values(starMapCache.regions).forEach(region => {
            if (region.solarSystemIDs) {
                region.solarSystemIDs.forEach(id => allSystemIds.add(id));
            }
            if (region.constellationIDs) {
                region.constellationIDs.forEach(id => allConstellationIds.add(id));
            }
        });
    }
    
    // Extract constellation IDs and their content
    if (starMapCache.constellations) {
        Object.keys(starMapCache.constellations).forEach(id => {
            allConstellationIds.add(parseInt(id));
        });
        
        Object.values(starMapCache.constellations).forEach(constellation => {
            if (constellation.solarSystemIDs) {
                constellation.solarSystemIDs.forEach(id => allSystemIds.add(id));
            }
        });
    }
    
    // Check if there's a direct solarSystems object
    if (starMapCache.solarSystems) {
        Object.keys(starMapCache.solarSystems).forEach(id => allSystemIds.add(parseInt(id)));
    }
    
    console.log(`Found ${allSystemIds.size} unique solar system IDs`);
    console.log(`Found ${allConstellationIds.size} unique constellation IDs`);
    console.log(`Found ${allRegionIds.size} unique region IDs`);
    
    console.log('Building message ID mappings...');
    
    // Build mappings from IDs to message IDs
    const systemIdToMessageId = new Map();
    const constellationIdToMessageId = new Map();
    const regionIdToMessageId = new Map();
    
    // Parse the labels from main localization
    if (mainLocalization.labels) {
        Object.entries(mainLocalization.labels).forEach(([messageId, entry]) => {
            if (entry.FullPath === 'Map/SolarSystems' && entry.label) {
                // Extract system ID from label like "solar_system_30000001"
                const match = entry.label.match(/solar_system_(\d+)/);
                if (match) {
                    const systemId = parseInt(match[1]);
                    if (allSystemIds.has(systemId)) {
                        systemIdToMessageId.set(systemId, parseInt(messageId));
                    }
                }
            } else if (entry.FullPath === 'Map/Constellations' && entry.label) {
                // Extract constellation ID from label like "constellation_20000001"
                const match = entry.label.match(/constellation_(\d+)/);
                if (match) {
                    const constellationId = parseInt(match[1]);
                    if (allConstellationIds.has(constellationId)) {
                        constellationIdToMessageId.set(constellationId, parseInt(messageId));
                    }
                }
            } else if (entry.FullPath === 'Map/Regions' && entry.label) {
                // Extract region ID from label like "region_10000001"
                const match = entry.label.match(/region_(\d+)/);
                if (match) {
                    const regionId = parseInt(match[1]);
                    if (allRegionIds.has(regionId)) {
                        regionIdToMessageId.set(regionId, parseInt(messageId));
                    }
                }
            }
        });
    }
    
    console.log(`Found ${systemIdToMessageId.size} solar system message ID mappings`);
    console.log(`Found ${constellationIdToMessageId.size} constellation message ID mappings`);
    console.log(`Found ${regionIdToMessageId.size} region message ID mappings`);
    
    console.log('Extracting names from English localization...');
    
    // Build final mappings from IDs to names
    const systemLabels = new Map();
    const constellationLabels = new Map();
    const regionLabels = new Map();
    
    // Skip the first two elements (language info) and get to the actual data
    const localizationData = enUsLocalization[1];
    
    // Extract system names
    systemIdToMessageId.forEach((messageId, systemId) => {
        if (localizationData[messageId]) {
            const localizationEntry = localizationData[messageId];
            if (Array.isArray(localizationEntry) && localizationEntry.length > 0) {
                const systemName = localizationEntry[0];
                if (systemName && systemName !== null) {
                    systemLabels.set(systemId, systemName);
                }
            }
        }
    });
    
    // Extract constellation names
    constellationIdToMessageId.forEach((messageId, constellationId) => {
        if (localizationData[messageId]) {
            const localizationEntry = localizationData[messageId];
            if (Array.isArray(localizationEntry) && localizationEntry.length > 0) {
                const constellationName = localizationEntry[0];
                if (constellationName && constellationName !== null) {
                    constellationLabels.set(constellationId, constellationName);
                }
            }
        }
    });
    
    // Extract region names
    regionIdToMessageId.forEach((messageId, regionId) => {
        if (localizationData[messageId]) {
            const localizationEntry = localizationData[messageId];
            if (Array.isArray(localizationEntry) && localizationEntry.length > 0) {
                const regionName = localizationEntry[0];
                if (regionName && regionName !== null) {
                    regionLabels.set(regionId, regionName);
                }
            }
        }
    });
    
    console.log(`Successfully extracted ${systemLabels.size} system labels`);
    console.log(`Successfully extracted ${constellationLabels.size} constellation labels`);
    console.log(`Successfully extracted ${regionLabels.size} region labels`);
    
    // Convert to plain objects for easier consumption
    const systemResult = {};
    systemLabels.forEach((name, id) => {
        systemResult[id] = name;
    });
    
    const constellationResult = {};
    constellationLabels.forEach((name, id) => {
        constellationResult[id] = name;
    });
    
    const regionResult = {};
    regionLabels.forEach((name, id) => {
        regionResult[id] = name;
    });
    
    // Write the results to separate JSON files
    const systemOutputPath = path.join(dataDir, 'system_labels.json');
    const constellationOutputPath = path.join(dataDir, 'constellation_labels.json');
    const regionOutputPath = path.join(dataDir, 'region_labels.json');
    
    fs.writeFileSync(systemOutputPath, JSON.stringify(systemResult, null, 2));
    fs.writeFileSync(constellationOutputPath, JSON.stringify(constellationResult, null, 2));
    fs.writeFileSync(regionOutputPath, JSON.stringify(regionResult, null, 2));
    
    console.log(`System labels saved to ${systemOutputPath}`);
    console.log(`Constellation labels saved to ${constellationOutputPath}`);
    console.log(`Region labels saved to ${regionOutputPath}`);
    
    // Also create a combined file
    const combinedResult = {
        systems: systemResult,
        constellations: constellationResult,
        regions: regionResult
    };
    
    const combinedOutputPath = path.join(dataDir, 'stellar_labels.json');
    fs.writeFileSync(combinedOutputPath, JSON.stringify(combinedResult, null, 2));
    console.log(`Combined labels saved to ${combinedOutputPath}`);
    
    // Show some examples
    console.log('\nExample system labels:');
    let count = 0;
    for (const [id, name] of systemLabels.entries()) {
        console.log(`  ${id}: ${name}`);
        count++;
        if (count >= 5) break;
    }
    
    console.log('\nExample constellation labels:');
    count = 0;
    for (const [id, name] of constellationLabels.entries()) {
        console.log(`  ${id}: ${name}`);
        count++;
        if (count >= 5) break;
    }
    
    console.log('\nExample region labels:');
    count = 0;
    for (const [id, name] of regionLabels.entries()) {
        console.log(`  ${id}: ${name}`);
        count++;
        if (count >= 5) break;
    }
    
    return combinedResult;
}

// Run the extraction if this script is executed directly
if (process.argv[1] && process.argv[1].endsWith('extract_system_labels.js')) {
    console.log('Starting stellar label extraction...');
    extractStellarLabels();
}

export { extractStellarLabels }; 