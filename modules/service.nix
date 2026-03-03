{inputs, ...}: let
  overlay = inputs.self.overlays.default;
in {
  flake.nixosModules.autarch-hardware = {
    config,
    lib,
    pkgs,
    ...
  }: let
    cfg = config.services.autarch-hardware;
  in {
    options.services.autarch-hardware = {
      enable = lib.mkEnableOption "autarch-hardware GPIO keyboard service";

      assignments = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [];
        example = ["1:space" "2:enter" "3:tab"];
        description = "List of GPIO-to-key assignments in GPIO:KEY format";
      };

      logDir = lib.mkOption {
        type = lib.types.path;
        default = "/var/log/autarch-hardware";
        description = "Directory for log files";
      };

      package = lib.mkOption {
        type = lib.types.package;
        default = pkgs.autarch-hardware;
        description = "The autarch-hardware package to use";
      };
    };

    config = lib.mkMerge [
      {
        nixpkgs.overlays = [overlay];
      }
      (lib.mkIf cfg.enable {
        systemd.services.autarch-hardware = {
          description = "Autarch Hardware GPIO Keyboard Service";
          wantedBy = ["graphical.target"];
          after = ["graphical.target"];

          environment = {
            AUTARCH_HARDWARE_LOG_PATH = cfg.logDir;
            DISPLAY = ":0";
            XAUTHORITY = "/home/nixos/.Xauthority";
          };

          serviceConfig = {
            Type = "simple";
            ExecStart = "${lib.getExe cfg.package} --assignments ${lib.concatStringsSep " " cfg.assignments}";
            Restart = "on-failure";
            RestartSec = 10;

            User = "nixos";
            Group = "users";
            SupplementaryGroups = ["input" "gpio"];

            LogsDirectory = "autarch-hardware";
            LogsDirectoryMode = "0755";
          };
        };
      })
    ];
  };
}
