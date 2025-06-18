import fs from "fs/promises";
import { Parser } from "pickleparser";

/**
 * Convert a pickle file to JSON using the pickleparser library
 * @param {string} pickleFilePath - Path to the input pickle file
 * @param {string} jsonFilePath - Path to the output JSON file
 */
export async function convertPickleToJson(pickleFilePath, jsonFilePath) {
  try {
    // Read the pickle file as a buffer
    const pickleData = await fs.readFile(pickleFilePath);
    
    // Parse the pickle data
    const parser = new Parser();
    const data = parser.parse(pickleData);
    
    // Convert to JSON and write to file
    const jsonData = JSON.stringify(data, null, 2);
    await fs.writeFile(jsonFilePath, jsonData, 'utf8');
    
    console.log(`Converted ${pickleFilePath} to ${jsonFilePath}`);
  } catch (error) {
    console.error(`Error converting ${pickleFilePath} to JSON:`, error);
    throw error;
  }
} 