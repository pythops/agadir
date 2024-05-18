default:
    @just --list

connect:
    @ssh localhost -p 2222 -o "UserKnownHostsFile=/dev/null" -o "StrictHostKeyChecking no"

dev:
    @cargo watch -c -w src -x run
