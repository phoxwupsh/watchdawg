# watchdawg

When using `auth_basic` module in Nginx, one certain problem is that it lacks session support, but sometimes we just need a simple HTTP basic authentication, rather than other complicated user management system. Thus, this project is to solve this problem, watchdawg offers a simple solution to maintain user sessions, and it also allows the use of existing `htpasswd` files. Besides being an authentication server, it can also be used as a reverse proxy with authentication, which allows it to work standalone.

## Features
- Just like `auth_basic`, simply works with HTTP basic authentication and htpasswd, but with session
- Authentication server for Nginx `auth_request`
- Standalone reverse proxy with authentication
- Supports `HTTP/1`, `HTTP/1.1`, `HTTP/2`
- Supports HTTPS

## Usage

watchdawg requires a config file to work, you can quickly get started with the template in the repository. For basic usage, you only need to choose which port to listen by `listen_port`, and the path to your htpasswd file by `htpasswd_path`. Name it `config.toml` and place it at the same directory with the watchdawg executable, then just run the executable. Or, you can also specify where the config file is like this

```
./watchdawg --config some/path/to/config.toml
```

> [!NOTE]
> For encryption algorithm of htpasswd, watchdawg currently only support bcrypt


### Authentication only

To use watchdawg as an authentication-only server, you need to set `enabled` in `[reverse_proxy]` section to `false` in the config file. Then, you need to specify the name of header containing the session ID that it to response to nginx (the default is `X-Auth-Token`). And then, you can configure you Nginx like this:

```
        location / {
            auth_request    /auth;
            auth_request_set $token $upstream_http_x_auth_token;
            add_header Set-Cookie $token;

            root   html;
            index  index.html index.htm;
        }

        location = /auth {
            internal;
            proxy_pass http://127.0.0.1:8080;
            proxy_pass_request_body off;
        }
```

In the above setting, 
1. `proxy_pass http://127.0.0.1:8080;` : Proxy `/auth` to the address it listening with `proxy_pass http://127.0.0.1:8080;` 
2. `auth_request_set $token $upstream_http_x_auth_token;`: to set the value of `X-Auth-Token` header (which is the session ID) to a variable `$token`. 
3. `add_header Set-Cookie $token;`: Pass `$token` to the user through cookies.

### Reverse proxy with authentication
watchdawg can be use as a reverse proxy, so it can work standalone without Nginx. To turn on reverse proxy mode, you need to set `enabled` in `[reverse_proxy]` section to `true` in the config file. Then, you need to specify `proxy_address` to the destination to forward all the requests.

### HTTPS
Both authentication-only and reverse proxy mode can use HTTPS. To turn on https, you need to set `enabled` in `[reverse_proxy]` section to `true` in the config file, and set `cert` and `key` to the path to your SSL/TLS certificate and private key.

### Session storage
The user sessions depend on cookies. In the [session] section of config file, you can specify the name of cookie that storing the session ID by `cookie_name`, and when to expire by `expire_time` (denoted in second). On the server side, the user sessions need to be stored somewhere, for now it supports 

| Where        | config value |
|--------------|--------------|
| In memory    | `memory`     |
| Redis        | `redis`      |

To switch where to store, you can set `storage` to coresponding config value in the above table. For Redis, you also need to set `redis_conn` to your connection information (For example `redis://user123:password456@127.0.0.1:6379/0`) of your redis server.

## Benchmark
I'm not sure how to benchmark a reverse proxy, so I simply benchmark authentication only mode. See the results.

## Planning
- [ ] More encryption algorithm for htpasswd (like apr1, sha-1)
- [ ] More session storage (maybe SQLite)
- [ ] Docker support
- [ ] Graceful shutdown
