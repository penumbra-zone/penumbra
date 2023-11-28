import path from 'path';
import { execSync } from 'child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'fs';

const TARGETS_WITHOUT_BIN = ['web', 'nodejs', 'bundler'];
const TARGETS_WITH_BIN = ['web-bin', 'nodejs-bin', 'bundler-bin'];

TARGETS_WITHOUT_BIN.forEach(wasmCompile);
TARGETS_WITH_BIN.forEach(wasmCompileBinary);

function wasmCompile(target: string): void {
  // Run wasm-pack for each target
  execSync(
    `wasm-pack build ../ --release --target ${target} --out-name index --out-dir publish/${target}`,
    {
      stdio: 'inherit',
    },
  );

  // Rename package to target-specific names
  const packageJsonPath = path.join(process.cwd(), `${target}/package.json`);
  const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf-8'));

  packageJson.name = `@penumbra-zone-test/wasm-${target}`;
  writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2), 'utf-8');

  // Without packing first, the .wasm's will not be included
  process.chdir(target);
  execSync('npm pack', { stdio: 'inherit' });

  // Publish to npm if flag provided
  if (process.argv.includes('--publish')) {
    execSync('npm publish --access public', { stdio: 'inherit' });
  }

  // Change working directory back to parent
  process.chdir('..');
};

function wasmCompileBinary(target: string): void {
  // Execute if 'bundle' flag is set
  if (process.argv.includes('--bundle')) {
    // Logically seperate the copy and build targets
    const buildTarget = target.replace('-bin', ''); 
    const copyTarget = target; 

    // Run wasm-pack for each target
    execSync(
      `wasm-pack build ../ --release --target ${buildTarget} --out-name index --out-dir publish/${copyTarget}`,
      {
        stdio: 'inherit',
      },
    );
    // Copy binary files to the package directory
    const binaryDir = path.join(process.cwd(), '../../crypto/proof-params/src/gen/');
    const targetPackageDir = path.join(process.cwd(), `${copyTarget}`);

    // Ensure the target directory exists
    if (existsSync(binaryDir)) {
      const targetBinaryDir = path.join(targetPackageDir, 'bin');
      if (!existsSync(targetBinaryDir)) {
        mkdirSync(targetBinaryDir);
      }

      // Copy binary files to the package directory
      const binaryFiles = [
        'delegator_vote_pk.bin', 
        'nullifier_derivation_pk.bin',
        'output_pk.bin',
        'spend_pk.bin',
        'swap_pk.bin',
        'swapclaim_pk.bin',
        'undelegateclaim_pk.bin'
      ]; 
      binaryFiles.forEach(file => {
        const sourcePath = path.join(binaryDir, file);
        const targetPath = path.join(targetBinaryDir, file);
        copyFileSync(sourcePath, targetPath);
      });
    } else {
      // Throw error if target directory doesn't exist
      throw new Error(`The directory ${binaryDir} does not exist.`);
    }

    // Rename package to target-specific names
    const packageJsonPath = path.join(process.cwd(), `${copyTarget}/package.json`);
    const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf-8'));

    // Check if the 'files' property in the generated package.json includes the 
    // bin directory, otherwise include it. Without this line, wasm-pack will
    // fail to bundle the binary proving keys inside the NPM package. 
    if (!packageJson.files.includes('bin')) {
      packageJson.files.push('bin');
    }
    packageJson.name = `@penumbra-zone-test/wasm-${copyTarget}`;
    writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2), 'utf-8');

    // Without packing first, the .wasm's will not be included
    process.chdir(copyTarget);
    execSync('npm pack', { stdio: 'inherit' });

    // Publish to npm if flag provided
    if (process.argv.includes('--publish')) {
      execSync('npm publish --access public', { stdio: 'inherit' });
    }

    // Change working directory back to parent
    process.chdir('..');
  }
};
