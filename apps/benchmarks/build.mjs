import esbuild from 'esbuild';
import tailwindcss from '@tailwindcss/postcss';
import fs from 'node:fs';
import postCssPlugin from '@deanc/esbuild-plugin-postcss';

const args = process.argv.slice(2);

const watch = args.includes('--watch');

/** @type {esbuild.BuildOptions} */
let options = {
  entryPoints: ['src/index.ts', 'src/index.html', 'src/worker.ts'],
  bundle: true,
  outdir: 'static/',
  loader: {
    '.html': 'copy'
  },
  plugins: [
    {
      name: 'rebuild-notify',
      setup(build) {
        let start;
        build.onStart(() => {
          start = Date.now();
          console.log('Building...');
        });

        build.onEnd((result) => {
          if (result.errors.length === 0) {
            const end = Date.now();
            const timeTaken = end - start;
            console.log('Build succeeded in:', timeTaken, 'ms');
          }
        });
      }
    },
    postCssPlugin({
      plugins: [
        tailwindcss({
          optimize: {
            minify: true
          }
        })
      ]
    })
  ]
};

if (watch) {
  const context = await esbuild.context({
    ...options,
    define: {
      IS_PRODUCTION: 'false'
    }
  });

  await context.watch();

  let { port } = await context.serve({
    servedir: 'static'
  });

  console.log(`Server running at http://localhost:${port}`);
} else {
  await esbuild.build({
    ...options,
    define: {
      IS_PRODUCTION: 'true'
    }
  });
}
