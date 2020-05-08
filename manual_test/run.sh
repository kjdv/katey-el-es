#!/bin/bash

set -e

((echo foo | ./tcp-echo-client.sh > /dev/null) && echo tcp-echo ok)
((./tcp-fib-client.sh > /dev/null) && echo tcp-fib ok)
((echo foo | ./katey-echo-client.sh > /dev/null) && echo katey-echo ok)
((./katey-fib-client.sh > /dev/null) && echo katey-fib ok)

((! (./katey-bad-server.sh > /dev/null 2>&1)) && echo katey-bad-server ok)
((! (./katey-bad-client.sh > /dev/null 2>&1)) && echo katey-bad-client ok)
echo done
