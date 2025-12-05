{
  config,
  pkgs,
  lib,
  ...
}:
let
  inherit (lib)
    types
    mkEnableOption
    mkOption
    mkIf
    literalExpression
    ;
  cfg = config.services.nixpkgs-build-failure-notifier;

  package = pkgs.callPackage ./package.nix { };
in
{
  options = {
    services.nixpkgs-build-failure-notifier = {
      enable = mkEnableOption (lib.mdDoc "nixpkgs-build-failure-notifier");

      user = mkOption {
        type = types.str;
        default = "nixpkgs-build-failure-notifier";
        description = lib.mdDoc "User account under which nixpkgs-build-failure-notifier runs.";
      };

      group = mkOption {
        type = types.str;
        default = "nixpkgs-build-failure-notifier";
        description = lib.mdDoc "Group under which nixpkgs-build-failure-notifier runs.";
      };

      configureDatabase = mkEnableOption "configure postgresql database using unix sockets";

      package = mkOption {
        type = types.package;
        default = package;
        defaultText = literalExpression "pkgs.nixpkgs-build-failure-notifier";
        description = lib.mdDoc ''
          nixpkgs-build-failure-notifier package to use.
        '';
      };

      jobsets = mkOption {
        type = types.listOf types.str;
        default = [ ];
        description = ''
          The jobsets to monitor.
        '';
        example = [ "nixpkgs:unstable" ];
      };

      jobs = mkOption {
        type = types.listOf types.str;
        default = [ ];
        description = ''
          The jobs to monitor.
        '';
        example = [
          "hello"
          "hydra"
        ];
      };

      maintainers = mkOption {
        type = types.listOf types.str;
        default = [ ];
        description = ''
          The maintainers whose packages to monitor.
        '';
        example = [
          "johndoe"
        ];
      };

      environment = mkOption {
        type = types.attrsOf types.anything;
        default = { };
        description = lib.mdDoc ''
          Structural nixpkgs-build-failure-notifier configuration.
          Refer to upstream's documentation for details and supported values.
        '';
        example = literalExpression ''
          {
            DB_URL = "postgresql:://localhost:5432";
          }
        '';
      };

      environmentFile = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = lib.mdDoc ''
          File containing settings to pass onto nixpkgs-build-failure-notifier.
          This is useful for secret configuration that should not be copied
          into the world-readable Nix store, for example, SMTP_PASSWORD.

          File must be in the following format:

          ```
          KEY=value
          ```
        '';
      };

      timerExpression = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = lib.mdDoc ''
          Schedule for the systemd timer. If null, no timer will be created.
          See https://www.freedesktop.org/software/systemd/man/latest/systemd.time.html
        '';
      };
    };
  };

  config = mkIf cfg.enable {
    services.postgresql = mkIf cfg.configureDatabase {
      enable = true;
      ensureDatabases = [ cfg.user ];
      ensureUsers = [
        {
          name = cfg.user;
          ensureDBOwnership = true;
          ensureClauses.login = true;
        }
      ];
    };

    systemd.services.nixpkgs-build-failure-notifier = {
      description = "nixpkgs-build-failure-notifier";
      after = [ "network.target" ] ++ lib.optionals cfg.configureDatabase [ "postgresql.target" ];
      requires = lib.optionals cfg.configureDatabase [ "postgresql.target" ];

      environment =
        (lib.optionalAttrs cfg.configureDatabase {
          DB_URL = "postgresql:///${cfg.user}?host=/run/postgresql";
        })
        // cfg.environment;

      serviceConfig = {
        Type = "oneshot";
        User = cfg.user;
        Group = cfg.group;
        ExecStart =
          let
            flags =
              (map (jobset: "--jobset=${jobset}") cfg.jobsets)
              ++ (map (job: "--job=${job}") cfg.jobs)
              ++ (map (maintainer: "--maintainer=${maintainer}") cfg.maintainers);
            cliFlags = lib.optionalString (flags != [ ]) " ${lib.escapeShellArgs flags}";
          in
          "${lib.getExe cfg.package}${cliFlags}";
        EnvironmentFile = [ cfg.environmentFile ];

        # systemd hardening
        NoNewPrivileges = true;
        SystemCallArchitectures = "native";
        RestrictAddressFamilies = [
          "AF_UNIX"
          "AF_INET"
          "AF_INET6"
        ];
        RestrictNamespaces = !config.boot.isContainer;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        ProtectControlGroups = !config.boot.isContainer;
        ProtectHostname = true;
        ProtectKernelLogs = !config.boot.isContainer;
        ProtectKernelModules = !config.boot.isContainer;
        ProtectKernelTunables = !config.boot.isContainer;
        LockPersonality = true;
        PrivateTmp = !config.boot.isContainer;
        PrivateDevices = true;
        PrivateUsers = true;
        RemoveIPC = true;

        SystemCallFilter = [
          "~@clock"
          "~@aio"
          "~@chown"
          "~@cpu-emulation"
          "~@debug"
          "~@keyring"
          "~@memlock"
          "~@module"
          "~@mount"
          "~@obsolete"
          "~@privileged"
          "~@raw-io"
          "~@reboot"
          "~@setuid"
          "~@swap"
        ];
        SystemCallErrorNumber = "EPERM";
      };
    };

    systemd.timers.nixpkgs-build-failure-notifier = mkIf (cfg.timerExpression != null) {
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = cfg.timerExpression;
      };
    };

    users.users = mkIf (cfg.user == "nixpkgs-build-failure-notifier") {
      nixpkgs-build-failure-notifier = {
        isSystemUser = true;
        group = cfg.group;
      };
    };

    users.groups = mkIf (cfg.group == "nixpkgs-build-failure-notifier") {
      nixpkgs-build-failure-notifier = { };
    };
  };
}
