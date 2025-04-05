# react-cursors

## Getting Started

To run locally

1. Make sure you have the ember link server running locally. Instructions can be found [here](../../README.md).
2. Install the dependencies with yarn or npm or whatever package manager you use

   ```sh
   yarn install
   ```

3. If needed update the port inside of [App.tsx](src/App.tsx) (it should match the port you used to run the docker image)

   ```tsx
   <EmberLinkProvider baseUrl="http://localhost:PORT_GOES_HERE">
     <ChannelProvider channelName="test">
       <Page />
     </ChannelProvider>
   </EmberLinkProvider>
   ```

4. Run the development server

   ```sh
   yarn dev
   ```

5. Open the URL vite outputs

6. See the magic!
