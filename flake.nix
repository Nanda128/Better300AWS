# `This file is an artifact from the original author's implementation`

{
	description = "A better M7 Route 300 express";
  
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, utils, naersk }: utils.lib.eachDefaultSystem (system: 
    let
      pkgs = nixpkgs.legacyPackages."${system}";
      naersk-lib = naersk.lib."${system}";
      package_name = "better300";
    in rec {

      # `nix build`
      packages."${package_name}" = naersk-lib.buildPackage {
        pname = "${package_name}";
        root = ./.;
        
        buildInputs = [
          pkgs.openssl
          pkgs.pkg-config
        ];
      };

      defaultPackage = packages."${package_name}";

      # `nix run`
      apps."${package_name}" = utils.lib.mkApp {
        drv = packages."${package_name}";
      };

      defaultApp = apps."${package_name}";

      # `nix develop`
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ rustc cargo ];
      };
      
      nixosModule = { lib, pkgs, config, ... }: 
        with lib; 
        let
          cfg = config.services."${package_name}";
        in { 
          options.services."${package_name}" = {
            enable = mkEnableOption "enable ${package_name}";
            
            database = mkOption rec {
              type = types.str;
              default = "database.db";
              example = default;
              description = "The path to the database";
            };
            
            host_port = mkOption rec {
              type = types.str;
              default = "127.0.0.1:8061";
              example = default;
              description = "host/port for the program";
            };
            
            
           # specific for teh program running
           user = mkOption rec {
              type = types.str;
              default = "${package_name}";
              example = default;
              description = "The user to run the service";
           };
           
           home = mkOption rec {
              type = types.str;
              default = "/etc/silver_${package_name}";
              example = default;
              description = "The home for the user";
           };
            
          };

          config = mkIf cfg.enable {
          
            users.groups."${cfg.user}" = { };
            
            users.users."${cfg.user}" = {
              createHome = true;
              isSystemUser = true;
              home = "${cfg.home}";
              group = "${cfg.user}";
            };
            
            systemd.services."${cfg.user}" = {
              description = "Better M7 Express";
              
              wantedBy = [ "multi-user.target" ];
              after = [ "network-online.target" ];
              wants = [ ];
              serviceConfig = {
                # fill figure this out in teh future
                #DynamicUser=true;
                User = "${cfg.user}";
                Group = "${cfg.user}";
                Restart = "always";
                ExecStart = "${self.defaultPackage."${system}"}/bin/better300 ${cfg.database} ${cfg.host_port}";
              };
            };
            
            # for updating the data
            systemd.services."${cfg.user}_update" = {
              description = "Better M7 Express Update Script";
              
              wantedBy = [ ];
              after = [ "network-online.target" ];
              serviceConfig = {
                Type = "oneshot";
                User = "${cfg.user}";
                Group = "${cfg.user}";
                ExecStart = "${self.defaultPackage."${system}"}/bin/get_data ${cfg.database}";
              };
            };
             
            systemd.timers."${cfg.user}_update" = {
              description="Run the update script for Better 300";
              
              wantedBy = [ "timers.target" ];
              partOf = [ "${cfg.user}_update.service" ];
              timerConfig = {
                OnCalendar = "*-*-* *:*:00";
                Unit = "${cfg.user}_update.service";
              };
            };
            
          };
          
        };
      
    });
}
