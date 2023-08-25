import path from 'path';
import { execSync } from 'child_process';
import { readFileSync, writeFileSync } from 'fs';

const targets = ['web', 'nodejs', 'bundler'];

targets.forEach(target => {
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
  packageJson.name = `@penumbra-zone/wasm-${target}`;
  writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2), 'utf-8');

  // Without packing first, the .wasm's will not be included
  process.chdir(target);
  execSync('npm pack', { stdio: 'inherit' });

  // Publish to npm
  execSync('npm publish --access public', { stdio: 'inherit' });

  // Change working directory back to parent
  process.chdir('..');
});
