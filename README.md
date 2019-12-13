# Medusa

Medusa helps you load test your API by sending many concurrent requests and recording timing statistics about each request.

Example:

```sh
$ cargo build
$ cargo run -- -t 300 -u 'https://jsonplaceholder.typicode.com/posts'
```

This will make 300 concurrent requests to the specified url and give an output similar the following snippet:

```
Successfully completed 293 requests
Avg response time: 5868ms
Min response time: 404ms
Max response time: 14669ms
```

## Future Work

* Read config from file
* Test multiple endpoints
* Add ability to make post requests by specifying data in body of request.
* Add more tests/docs
