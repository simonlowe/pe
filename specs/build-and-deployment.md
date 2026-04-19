# Build

## Executable build

There should be build scripts in the root of the project taht may be used by a developer working locally

One of the build scripts should build targets for all three plaforms:

- macos arm
- windows x64
- linux x64

That script should check the operating system of the developer's machine. It should then check for the presence of folder at ~/bin   if the folder exists, it should copy the appropriate built executable to that folder. 

## Github Workflows

This project will eventually be stored in github. It should have a .github folder and workflows as follows:

### build.yml

This workflow should build, test, apply code security checks, code qualty checks and package an artifact for each build target.  An e2e integraton test step will be included but wil not be executed unless an input is set. This workflow will be triggered by VCS commit, or by manual workflow dispatch

### deploy.yml

This workflow should be capable of downloading the build artifact and deploying it to servers by SSH. A server name, server path, and name of a github sceret for the SSH private key should be passed in as inputs. This workflow should eb triggered bya. manual dispatch
