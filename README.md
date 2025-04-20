# feedback

This is a very simple backend service for a contact/feedback form on a website. It forwards messages over the [Apprise](https://github.com/caronc/apprise-api) API.

Examples where I use it:

- [AutoPTT](https://autoptt.com/contact/)
- [Keybind Practice](https://autoptt.com/keys/) (on the "About" tab)
- [My Personal Site](https://virtala.dev/contact/)

## Install & Run

Grab a release binary. Create a `config.yml` (see below).  Run the server with

```sh
./feedback
```

It will search for `config.yml` in the current working directory by default. You can change it with the environment variable `FEEDBACK_CONFIG_PATH`.

## API Usage

Javascript example

```js
await fetch("https://feedback.example.com", {
  method: "POST",
  headers: {
    "content-type": "application/json",
  },
  body: JSON.stringify({
    subject: "My App", // required
    source: location.origin, // required
    message: "Hello there!", // required
    email: "user@example.com", // optional
  }),
})
```

## Configuration
### Stateful endpoint

```yml
addr: 127.0.0.1:3000
apprise:
  url: https://apprise.example.com/notify/example
```

### Stateless endpoint

```yml
addr: 127.0.0.1:3000
apprise:
  url: https://apprise.example.com/notify
  stateless_urls: ntfys://user:pass@ntfy.example.com/topic
```

### Extra headers

In case you need to add extra headers (eg. for auth purposes) use `apprise.headers`:

```yml
apprise:
  headers:
    some_header: some_value
    another_header: another_value
```
