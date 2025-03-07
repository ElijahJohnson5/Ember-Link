# javascript-live-cursors

## Getting Started

To run locally

1. Make sure you have the ember link server running locally. Instructions can be found [here](../../README.md).
2. Install the dependencies with yarn or npm or whatever package manager you use

   ```
   yarn install
   ```

3. If needed update the port inside of [app.ts](app.ts) (it should match the port you used to run the docker image)

   ```
   const client = createClient({
     baseUrl: 'http://localhost:PORT_GOES_HERE'
   });
   ```

4. Build the package

   ```
   yarn build
   ```

5. Open the index.html in the static folder in multiple browsers and see it working!
