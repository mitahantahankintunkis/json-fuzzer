# HAProxy rate limitation bypass
HAProxy is a popular reverse proxy with over a billion downloads in [Docker Hub](https://hub.docker.com/_/haproxy) described as "a free, very fast and reliable reverse-proxy offering high availability, load balancing, and proxying for TCP and HTTP-based applications"\[1]. One of its features is to act as a rate limiter between a client and a server. As an example, it can be configured to rate-limit requests by dropping those that match a certain IP address, header, URL, or body parameter after too many have been received within a timespan.

Internally, HAProxy uses [mjson](https://github.com/cesanta/mjson)\[3] to parse JSON data inside request bodies and headers. When encountering duplicate keys, mjson uses only the first duplicate key while ignoring the rest. While this functionality is correct according to the latest JSON specification\[4], it differs from the industry standard of using the last duplicate key inside JSON objects. Additionally, msjon does not parse unicode encoded values in strings.

Due to these discrepancies, the JSON object `{"rol\u0065":"admin","role":"user","role":"admin"}` gets parsed differently by mjson and the vast majority of other JSON parsers. While msjon believes that the key `role` has a value of `user`, almost all other parsers believe the value to be `admin`. This enables us to affect how HAProxy routes traffic when it is dependent on JSON values.

## Hypothetical scenario
An API is behind HAProxy, which combines multiple different API's to a single externally accessible endpoint. One of these API's handles user logins. To prevent password guessing and brute forcing, HAProxy limits the number of login attempts for a single username from the same IP address. If the number of login attempts exceeds five within a 60 second window, the proxy denies additional requests with HTTP status code 429: HTTP_TOO_MANY_REQUESTS.

Due to the API only accepting JSON data inside the POST request, HAProxy gets the username used for the rate limiting from JSON data.
Using the discrepancies described above, we can formulate the login request as `{"usernam\u0065":"<actual>", "username":"<haproxy>", "username":"<actual>", "password":"<password>"}`. In this way, HAProxy and the API disagree on what the requested username is. Thus we can bypass the rate limiting by sending random information in the second field, while sending the actual username in the first and third fields.

To run the hypothetical scenario, install docker and run the following commands (In linux or WSL):

```
docker compose up --build -d
./expected_behavior.sh
./bypass.sh
```

This sets up the following architecture:

![HAProxy architecture](paper/figures/haproxy-architecture.png "HAProxy architecture")

*Figure 1: Hypothetical scenario architecture*

The two bash scripts showcase the vulnerability. Their output can be seen in the figure below.

![HAProxy rate limiting bypass](paper/figures/haproxy-ratelimit-bypass.png "HAProxy rate limiting bypass")

*Figure 2: Output from two password brute forcing bash scripts; expected_behavior.sh and bypass.sh. The first shows rate limiting being applied, while the second shows how it can be bypassed.*

## References:
\[1] [https://hub.docker.com/_/haproxy](https://hub.docker.com/_/haproxy)

\[2] [https://github.com/haproxy/haproxy](https://github.com/haproxy/haproxy)

\[3] [https://github.com/cesanta/mjson](https://github.com/cesanta/mjson)

\[4] [https://datatracker.ietf.org/doc/html/rfc825](https://datatracker.ietf.org/doc/html/rfc825)
