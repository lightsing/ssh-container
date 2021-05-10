# ssh-container

This is a login-shell replacement to allocate container for user.

## Usage

Make sure you have a linux server, rust toolchain, a proper docker installation.

Execute `make install` to compile and install to `/opt/ssh-container`.

Create a new user with login shell of `/opt/ssh-container`. Add the user to `docker` group.

Now, you can log in with the user, and it will start a `ubunt:focal` container.
The container will be deleted before the ssh session is going to close.

Currently, it works with docker, and it's only a PoC (proof of concept).
It does not limit the container resource, be careful there will be a potential DoS vulnerability for open access.

## TODO

- [ ] Kubernetes support (then it works with cluster)
- [ ] Admin/Configurable Login Controller (Auth/Image selection)
- [ ] Optional persistent container (Require auth capability)
- [ ] scp support (scp from user into container)