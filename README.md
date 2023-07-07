# signserver

A containerized code signing HTTP server, powered by [osslsigncode](https://github.com/mtrojnar/osslsigncode/)

## Usage

__WARNING__: Make sure you add a reverse proxy with some sort of authentication.

1. Start the container. It is recommended that you use TPM backed code signing certificates (the container is built with tpm2-pkcs11), but this example uses a file:

    `podman run -v $(pwd)/wdktestcert.pfx:/wdktestcert.pfx:z -p 8080:8080 -e HOST=0.0.0.0 -e 'ARGS=-pkcs12$/wdktestcert.pfx$-pass$password$-h$sha256$-ph$-ts$http://timestamp.digicert.com' ghcr.io/namazso/signserver:latest `

2. Send a POST request to http://localhost:8080/sign with the file to sign as body. 

3. Receive back the signed file.

