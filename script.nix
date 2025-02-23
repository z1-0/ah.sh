{ pkgs, ... }:

pkgs.writeShellScriptBin "envshell" ''
    set -e

    show_help() {
      cat << EOF
  Usage: envshell [OPTIONS] [ARGUMENT]

  Options:
    -s, --shell [ARG]     Enter development shell
                          Optional ARG specifies a specific flake output
    
    -t, --template [ARG]  Use flake template, including justfile and treefmt
                          Optional ARG specifies a specific flake output
    
    -h, --help            Show this help message

  Examples:
    envshell -s              # Enter shell with default configuration
    envshell -s python       # Enter shell with python configuration
    envshell -t rust         # Use rust template
  EOF
    }

    # BASE_URL="sourcehut:~fangzirong/envshell"
    BASE_URL="github:6iovan/envshell"

    case "$1" in
      -s|--shell)
        if [ -n "$2" ]; then
          exec nix develop --no-pure-eval "$BASE_URL#$2"
        else
          exec nix develop --no-pure-eval "$BASE_URL"
        fi
        ;;

      -t|--template)
        if [ -n "$2" ]; then
          nix flake init -t "$BASE_URL#$2"
        else
          nix flake init -t "$BASE_URL"
        fi
        ;;

      -h|--help)
        show_help
        ;;

      *)
        echo "Error: Unknown option '$1'"
        echo "Use -h or --help to see usage information"
        exit 1
        ;;
    esac
''
