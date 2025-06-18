import fs from "fs/promises";

/**
 * Check if a file is a valid SQLite database by reading the header
 * @param {string} filePath - Path to the file to check
 * @returns {Promise<boolean>} - True if the file is a valid SQLite database
 */
export async function isValidSQLite(filePath) {
  try {
    // SQLite files start with "SQLite format 3\0"
    const buffer = await fs.readFile(filePath, { flag: 'r' });
    
    if (buffer.length < 16) {
      return false;
    }
    
    const sqliteHeader = "SQLite format 3\0";
    const headerBytes = buffer.subarray(0, 16);
    const headerString = headerBytes.toString('ascii');
    
    return headerString === sqliteHeader;
  } catch (error) {
    console.error(`Error checking SQLite validity for ${filePath}:`, error);
    return false;
  }
} 