# Ember Link

An open-source platform that enables developers to effortlessly add real-time collaboration features to their applications. With full self-hosting capabilities, this project allows complete control over your collaboration infrastructure. The included SDKs make it incredibly easy to integrate real-time functionality, providing a seamless setup experience. Whether you're building a team collaboration tool, a shared workspace, or any real-time application, this solution is fully customizable, easy to deploy, and 100% open-source.

## Getting Started

The easiest way to get started is to run the latest ember-link docker image locally and then run an example.

1. Download the docker image
   ```
   docker pull emberlinkio/ember-link:latest
   ```
2. Run the docker image
   This will run the docker image on port 9000 and expose it so you can connect to it from one of the examples. It also sets the `ALLOW_UNAUTHORIZED` flag to true for an easier time running the examples.
   ```
   docker run -d -p 9000:9000 --env PORT=9000 --env HOST=0.0.0.0 --env ALLOW_UNAUTHORIZED=true emberlinkio/ember-link:latest
   ```
3. Clone this repo into a directory you want it in
   ```
   git clone https://github.com/ElijahJohnson5/Ember-Link.git
   ```
4. Find an example in the [examples folder](examples) you want to run and follow the instructions in the README for that example.

### Server Config

There are several environment variables you can set to change the behavior of the docker image for the ember-link server.

| Env Variable Name       | Description                                                                                          | Required | Default Value |
| ----------------------- | ---------------------------------------------------------------------------------------------------- | -------- | ------------- |
| HOST                    | The host address the server should run on                                                            | :x:      | 127.0.0.1     |
| PORT                    | The port the server should run on                                                                    | :x:      | 9000          |
| ALLOW_UNAUTHORIZED      | Whether or not to allow unauthorized connection to the server (without JWT's)                        | :x:      | false         |
| WEBHOOK_URL             | The url to send webhook notifications to                                                             | :x:      |               |
| JWT_SIGNER_KEY          | The key used for singing JWT                                                                         | :x:      |               |
| JWT_SIGNER_KEY_ENDPOINT | The endpoint to get the JWT_SIGNER_KEY for different tenants (only used with multi tenant operation) | :x:      |               |
| STORAGE_ENDPOINT        | The endpoint to load and save storage data to                                                        | :x:      |               |
