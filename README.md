# nixpkgs-build-failure-notifier

CLI tool that checks Hydra for build failures of a given package
or a maintainer's packages.
It can then send an email report if there are failures.

## Usage

```
$ nixpkgs-build-failure-notifier --help
Usage: nixpkgs-build-failure-notifier [OPTIONS] --db-url <DB_URL> --smtp-host <SMTP_HOST> --smtp-username <SMTP_USERNAME> --smtp-password <SMTP_PASSWORD> --smtp-from <SMTP_FROM> --smtp-to <SMTP_TO>

Options:
      --jobset <JOBSETS>               The jobsets where the given jobs will be searched for
  -j, --job <JOBS>                     The jobs to monitor
      --system <SYSTEMS>               The systems (e.g., x86_64-linux) to monitor jobs to. Defaults to nixpkgs' default systems (x86_64-linux, aarch64-linux, x86_64-darwin, and aarch64-darwin) [default: x86_64-linux aarch64-linux x86_64-darwin aarch64-darwin]
      --maintainer <MAINTAINERS>       Maintainers whose packages should also be checked
      --db-url <DB_URL>                Connection string to the PostgreSQL database [env: DB_URL=]
      --smtp-host <SMTP_HOST>          Hostname of the SMTP server [env: SMTP_HOST=]
      --smtp-username <SMTP_USERNAME>  Username to use when connecting to the SMTP server [env: SMTP_USERNAME=]
      --smtp-password <SMTP_PASSWORD>  Password to use when connecting to the SMTP server [env: SMTP_PASSWORD=]
      --smtp-from <SMTP_FROM>          Email address to send emails from [env: SMTP_FROM=]
      --smtp-to <SMTP_TO>              Destination address of the email notifications [env: SMTP_TO=]
  -h, --help                           Print help
  -V, --version                        Print version
```
