# i want to ping the ipv4 internet


## Plan

```rust
for i in 0..=255 {
    for j in 0..=255 {
        for k in 0..=255 {
            for l in 0..=255 {
                let ip_addr = format!("{}.{}.{}.{}", i, j, k, l)
                let time_to_response: Optional<Duration> = ping(ip_addr);
                match time_to_response {
                    Some(time) => {
                        // write to sqlite
                    },
                    None => {
                        // just move on
                    }
                }
            }
        }
    }
}
```

But that'll take a while. There are $256^4$ ip addresses, which is nearly 4.3 billion. Waiting for a response for each would take, at worst, 4.3 billion seconds, which is roughly 90 years, and at that point the internet will probably have transitioned to ipv6, so I'd have to just start all over again.

## Plan v2: just like make a lot of threads, man
