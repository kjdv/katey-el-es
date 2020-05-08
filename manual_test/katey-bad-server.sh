#!/bin/bash

exec katey-client --root=other-root-cert.pem localhost:5000 $@
