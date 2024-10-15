# Benchmark for authentication only mode

## Hardware
- CPU: Intel Core i7 14700 20C28T
- RAM: 64GB 5600MT/s

## Benchmark: In memory
watchdawg config file:
```toml
# config.toml
listen_address = "127.0.0.1"
listen_port = 8080
htpasswd_path = "htpasswd"
auth_return_header_name = "X-Auth-Token"
debug = false

[reverse_proxy]
enabled = false

[session]
cookie_name = "session_id"
session_expire = 86400
storage = "memory"
```

Benchmark command:
```shell
vegeta attack -targets req.txt -rate 0 -format http -duration 10s -output auth.bin -max-workers 8
```

### No authentication and no session

#### HTTP request
```
# req.txt
GET http://127.0.0.1:8080
```

#### Result
```
Requests      [total, rate, throughput]         804036, 80400.92, 0.00
Duration      [total, attack, wait]             10s, 10s, 0s
Latencies     [min, mean, 50, 90, 95, 99, max]  0s, 90.367µs, 0s, 514.933µs, 519.242µs, 1ms, 1.712ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           0.00%
Status Codes  [code:count]                      401:804036
Error Set:
401 Unauthorized
```

### Valid session

The session ID is retrived first from a browser

#### HTTP request
```
# req.txt
GET http://127.0.0.1:8080
Cookie: session_id=d42f4258-ea33-498e-b7d2-3d5617b2a658
```

#### Result
```
Requests      [total, rate, throughput]         1056079, 105599.46, 105599.46
Duration      [total, attack, wait]             10.001s, 10.001s, 0s
Latencies     [min, mean, 50, 90, 95, 99, max]  999.4µs, 62.82µs, 0s, 13.046µs, 521.432µs, 1.001ms, 2.506ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           100.00%
Status Codes  [code:count]                      200:1056079
Error Set:
```

### Invalid session

#### HTTP request
```
# req.txt
GET http://127.0.0.1:8080
Cookie: session_id=some_invalid_session_id
```

### Result
```
Requests      [total, rate, throughput]         1007805, 100770.82, 0.00
Duration      [total, attack, wait]             10.001s, 10.001s, 0s
Latencies     [min, mean, 50, 90, 95, 99, max]  1.001ms, 65.292µs, 0s, 2.542µs, 522.247µs, 1.001ms, 2.146ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           0.00%
Status Codes  [code:count]                      401:1007805
Error Set:
401 Unauthorized
```

### Valid authentication

#### HTTP request
```
# req.txt
GET http://127.0.0.1:8080
Authorization: Basic dXNlcjoxMjM=
```

#### Result
```
Requests      [total, rate, throughput]         1539, 153.49, 152.77
Duration      [total, attack, wait]             10.074s, 10.027s, 46.928ms
Latencies     [min, mean, 50, 90, 95, 99, max]  45.905ms, 52.155ms, 47.272ms, 83.324ms, 84.28ms, 84.768ms, 85.471ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           100.00%
Status Codes  [code:count]                      200:1539
Error Set:
```

## Benchmark: Redis
watchdawg config file:
```toml
# config.toml
listen_address = "127.0.0.1"
listen_port = 8080
htpasswd_path = "htpasswd"
auth_return_header_name = "X-Auth-Token"
debug = false

[reverse_proxy]
enabled = false

[session]
cookie_name = "session_id"
session_expire = 86400
storage = "redis"
redis_conn = "redis://127.0.0.1:6379/1"
```

Benchmark command:
```shell
vegeta attack -targets req.txt -rate 0 -format http -duration 10s -output auth.bin -max-workers 8
```

### No authentication and no session

#### HTTP request
```
# req.txt
GET http://127.0.0.1:8080
```

#### Result
```
Requests      [total, rate, throughput]         1027252, 102733.35, 0.00
Duration      [total, attack, wait]             10s, 9.999s, 1.001ms
Latencies     [min, mean, 50, 90, 95, 99, max]  1.001ms, 63.472µs, 0s, 24.637µs, 522.116µs, 1.001ms, 1.721ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           0.00%
Status Codes  [code:count]                      401:1027252
Error Set:
401 Unauthorized
```

### Valid session

The session ID is retrived first from a browser

#### HTTP request
```
# req.txt
GET http://127.0.0.1:8080
Cookie: session_id=2f266008-0968-409d-bc28-070c2062c6b8
```

#### Result
```
Requests      [total, rate, throughput]         238674, 23868.29, 23867.05
Duration      [total, attack, wait]             10s, 10s, 520µs
Latencies     [min, mean, 50, 90, 95, 99, max]  520µs, 328.624µs, 502.976µs, 585.634µs, 605.431µs, 647.539µs, 1.629ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           100.00%
Status Codes  [code:count]                      200:238674
Error Set:
```

### Invalid session

#### HTTP request
```
# req.txt
GET http://127.0.0.1:8080
Cookie: session_id=some_invalid_session_id
```

### Result
```
Requests      [total, rate, throughput]         248685, 24868.44, 0.00
Duration      [total, attack, wait]             10s, 10s, 115.8µs
Latencies     [min, mean, 50, 90, 95, 99, max]  115.8µs, 314.696µs, 502.822µs, 585.532µs, 604.51µs, 638.872µs, 1.646ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           0.00%
Status Codes  [code:count]                      401:248685
Error Set:
401 Unauthorized
```

### Valid authentication

#### HTTP request
```
# req.txt
GET http://127.0.0.1:8080
Authorization: Basic dXNlcjoxMjM=
```

#### Result
```
Requests      [total, rate, throughput]         1703, 170.30, 169.54
Duration      [total, attack, wait]             10.045s, 10s, 44.538ms
Latencies     [min, mean, 50, 90, 95, 99, max]  40.632ms, 47.071ms, 44.604ms, 57.402ms, 62.498ms, 68.147ms, 85.231ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           100.00%
Status Codes  [code:count]                      200:1703
Error Set:
```