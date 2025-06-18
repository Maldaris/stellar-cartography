import "dotenv/config";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { promisify } from "util";
import readline from "readline";
import { isValidSQLite } from "./utils/is_valid_sqlite.js";
import { convertPickleToJson } from "./utils/pickle_to_json.js";

const copyFile = promisify(fs.copyFile);

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const { RES_FILE_INDEX_PATH, RES_FILES_BASE_DIR } = process.env;

// Ensure the environment variables are set
if (!RES_FILE_INDEX_PATH || !RES_FILES_BASE_DIR) {
  throw new Error("Environment variables RES_FILE_INDEX_PATH and RES_FILES_BASE_DIR must be set");
}

const dataBaseDir = path.join(__dirname, "data");
const destRawDir = path.join(dataBaseDir, "raw");
const destSqliteDir = path.join(dataBaseDir, "sqlite");
const destSchemaDir = path.join(dataBaseDir, "schema");
const destFsdBinaryDir = path.join(dataBaseDir, "fsdbinary/src");
const destStaticDir = path.join(dataBaseDir, "static");
const destPickleDir = path.join(dataBaseDir, "pickle");
const destJsonDir = path.join(dataBaseDir, "json");

// Ensure each destination directories exists
[
  dataBaseDir,
  destRawDir,
  destSqliteDir,
  destSchemaDir,
  destFsdBinaryDir,
  destStaticDir,
  destPickleDir,
  destJsonDir,
].forEach((dir) => {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
});

console.log(`Reading ResFile index from: ${RES_FILE_INDEX_PATH}`);
console.log(`ResFiles base directory: ${RES_FILES_BASE_DIR}`);

// Create a read stream for the res index file
const readStream = fs.createReadStream(RES_FILE_INDEX_PATH);
const rl = readline.createInterface({
  input: readStream,
  crlfDelay: Infinity,
});

// Regex to extract {filename}, {filetype}, {source_dir}, and {source_filename}
const regex = /^(.*?)([^\/]+)\.([^,]+),([^\/]+\/[^,]+)(.*)$/;

let processedCount = 0;
let pickleCount = 0;

rl.on("line", async (line) => {
  const match = line.match(regex);
  if (!match) return;

  const [_, respath, filename, filetype, sourceRelativePath] = match;
  const sourceFilePath = path.join(RES_FILES_BASE_DIR, sourceRelativePath);

  processedCount++;
  if (processedCount % 1000 === 0) {
    console.log(`Processed ${processedCount} files...`);
  }

  switch (filetype) {
    case "static":
      await processStaticFile(filename, sourceFilePath);
      break;
    case "schema":
      await processSchemaFile(filename, sourceFilePath);
      break;
    case "fsdbinary":
      await processFsdBinaryFile(filename, sourceFilePath);
      break;
    case "pickle":
      pickleCount++;
      console.log(`Processing pickle file #${pickleCount}: ${filename}`);
      await processPickleFile(filename, sourceFilePath);
      break;
    default:
      await defaultCopyFile(
        filename,
        filetype,
        sourceFilePath,
        respath.replace("res:", "")
      );
      break;
  }
});

async function processStaticFile(filename, sourceFilePath) {
  const destSourceFile = path.join(destStaticDir, `${filename}.static`);

  // Copy the static file to the destination directory
  await copyFile(sourceFilePath, destSourceFile);
  console.log(`Copied ${sourceFilePath} to ${destSourceFile}`);

  const isValid = await isValidSQLite(sourceFilePath);

  if (isValid) {
    const destSqliteFile = path.join(destSqliteDir, `${filename}.sqlite`);
    await copyFile(sourceFilePath, destSqliteFile);
    console.log(`Copied ${sourceFilePath} to ${destSqliteFile}`);
  } else {
    console.log(`Invalid SQLite file: ${sourceFilePath}`);
  }
}

async function processSchemaFile(filename, sourceFilePath) {
  const destSourceFile = path.join(destSchemaDir, `${filename}.schema`);
  await copyFile(sourceFilePath, destSourceFile);
  console.log(`Copied ${sourceFilePath} to ${destSourceFile}`);
}

async function processFsdBinaryFile(filename, sourceFilePath) {
  const destSourceFile = path.join(destFsdBinaryDir, `${filename}.fsdbinary`);
  await copyFile(sourceFilePath, destSourceFile);
  console.log(`Copied ${sourceFilePath} to ${destSourceFile}`);
}

async function processPickleFile(filename, sourceFilePath) {
  const destPickleFilePath = path.join(destPickleDir, `${filename}.pickle`);
  const destJsonFilePath = path.join(destJsonDir, `${filename}.json`);

  try {
    // Copy the pickle file to the destination directory
    await copyFile(sourceFilePath, destPickleFilePath);
    console.log(`Copied ${sourceFilePath} to ${destPickleFilePath}`);

    // Convert the pickle file to json
    await convertPickleToJson(destPickleFilePath, destJsonFilePath);
    
    // Check if this is the starmapcache file we're looking for
    if (filename.includes("starmapcache") || filename.includes("starmap")) {
      console.log(`⭐ Found starmap data: ${destJsonFilePath}`);
    }
  } catch (error) {
    console.error(`Error processing pickle file ${filename}:`, error);
  }
}

async function defaultCopyFile(
  filename,
  filetype,
  sourceFilePath,
  respath
) {
  const dir = path.join(destRawDir, respath);

  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }

  const destFilePath = path.join(dir, `${filename}.${filetype}`);
  await copyFile(sourceFilePath, destFilePath);
  console.log(`Copied ${sourceFilePath} to ${destFilePath}`);
}

rl.on("close", () => {
  console.log(`\nExtraction complete!`);
  console.log(`Total files processed: ${processedCount}`);
  console.log(`Pickle files found: ${pickleCount}`);
  console.log(`\nGenerated data directories:`);
  console.log(`- JSON files: ${destJsonDir}`);
  console.log(`- SQLite files: ${destSqliteDir}`);
  console.log(`- Pickle files: ${destPickleDir}`);
});

rl.on("error", (error) => {
  console.error("Error reading ResFile index:", error);
}); 