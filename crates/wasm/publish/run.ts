import path from 'path';
import { execSync } from 'child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'fs';

const TARGETS = ['web', 'nodejs', 'bundler'];

TARGETS.forEach(target => {
  // Run wasm-pack for each target
  execSync(
    `wasm-pack build ../ --release --target ${target} --out-name index --out-dir publish/${target}`,
    {
      stdio: 'inherit',
    },
  );

  if (target === 'bundler') {
    // Copy binary files to the package directory
    const binaryDir = path.join(process.cwd(), '../../crypto/proof-params/src/gen/');
    const targetPackageDir = path.join(process.cwd(), `${target}`);

    // Ensure the target binary directory exists
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
    }
  }

  // Rename package to target-specific names
  const packageJsonPath = path.join(process.cwd(), `${target}/package.json`);
  const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf-8'));
  if (!packageJson.files.includes('bin')) {
    packageJson.files.push('bin');
  }
  packageJson.name = `@penumbra-zone/wasm-${target}`;
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
});
