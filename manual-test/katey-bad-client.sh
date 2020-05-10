#!/bin/bash

exec katey-client --root=root-cert.pem --cert=other-client-cert.pem --key=other-client-key.pem localhost:5001 $@
