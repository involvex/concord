#!/usr/bin/env node

const { spawn } = require('child_process');
const { existsSync } = require('fs');
const { join } = require('path');

const projectRoot = __dirname;
const binaryName = process.platform === 'win32' ? 'concord.exe' : 'concord';
const releaseBinary = join(projectRoot, '..', 'target', 'release', binaryName);
const debugBinary = join(projectRoot, '..', 'target', 'debug', binaryName);

function getBinaryPath() {
  if (existsSync(releaseBinary)) {
    return releaseBinary;
  }
  if (existsSync(debugBinary)) {
    return debugBinary;
  }
  return null;
}

function main() {
  let binaryPath = getBinaryPath();

  if (!binaryPath) {
    console.error('Concord binary not found. Building release version...');
    const buildProcess = spawn('cargo', ['build', '--release'], {
      cwd: join(projectRoot, '..'),
      stdio: 'inherit',
      shell: true
    });

    buildProcess.on('close', (code) => {
      if (code !== 0) {
        console.error('Failed to build concord');
        process.exit(1);
      }
      binaryPath = releaseBinary;
      runBinary(binaryPath);
    });
    return;
  }

  runBinary(binaryPath);
}

function runBinary(path) {
  const child = spawn(path, process.argv.slice(2), {
    stdio: 'inherit',
    shell: process.platform === 'win32'
  });

  child.on('close', (code) => process.exit(code));
}

main();