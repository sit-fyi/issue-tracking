# sit-import

This tool allows to import issues (and merge requests) from external sources
to [SIT](http://sit-it.org), thus enabling seamless migration.

Currently supported sources:

* GitHub

That's it :)

## Status

Early version, published for testing. Certain types of data is **not imported
yet** and likely has bugs (check with sit-import's issues).

## How to build

You will need Rust 1.24+, openssl, pkgconfig. To build:

```
cargo build --release
```

You can now put `target/release/sit-import` into your `PATH`.

## Runnning

### Importing from GitHub

 Firstly, [create a personal token](https://github.com/settings/tokens). Then, create a config
 file (say, `import.json`) and put it there:

 ```json
 {
   "github": {
      "token": "<TOKEN>"
   }
}
```

Initialize a target repo by running `sit init` (it will put it into `.sit`) or `sit -r DEST init` (it will put it into `DEST`).

Run `sit-import`:

```
sit-import [-r DEST] -c import.json https://github.com/OWNER/REPO
```

Depending on the size of the project, your bandwidth and other parameters, it might take a while.

If you are satisifed with your test run, it is suggested that you disable
access to your issues/pull requests ([temporary interaction limit](https://github.com/blog/2370-introducing-temporary-interaction-limits( feature might come in handy), make sure other
collaborators/admins are staying off the issues and do a final run, ensuring
nothing is lost in the transition.

