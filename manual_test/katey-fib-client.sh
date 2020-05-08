#!/bin/bash

exec katey-client --root=root-cert.pem --cert=client-cert.pem --key=client-key.pem localhost:5001 $@
