# Sourced by openaec-start before the application starts.
export OPENAEC_FRAME_CONTAINER=1

if [[ -n ${OPENAEC_LANG:-} ]]; then
  frame_locale=${OPENAEC_LANG,,}
  frame_locale=${frame_locale%%[-_.]*}
  case "$frame_locale" in
    nl|en|de|tr) export OPENAEC_FRAME_LOCALE="$frame_locale" ;;
    *) echo 'Unsupported OPENAEC_LANG' >&2; exit 1 ;;
  esac
fi
case "${OPENAEC_THEME:-}" in
  ''|default|light|dark|blue|amber-navy|warm-ember|highContrast) ;;
  *) echo 'Unsupported OPENAEC_THEME' >&2; exit 1 ;;
esac

if [[ -n ${OPENAEC_OPEN:-} ]]; then
  candidate=$OPENAEC_OPEN
  [[ $candidate == /* ]] || candidate="/openaec/$candidate"
  canonical=$(realpath -e -- "$candidate") || { echo 'OPENAEC_OPEN does not exist' >&2; exit 1; }
  if [[ $canonical != /openaec/* || ${canonical,,} != *.ofs || ! -f $canonical || ! -r $canonical ]]; then
    echo 'OPENAEC_OPEN must be a readable .ofs file inside /openaec' >&2
    exit 1
  fi
  export OPENAEC_OPEN="$canonical"
fi

if [[ -n ${OPENAEC_MCP_TOKEN_FILE:-} ]]; then
  [[ -r $OPENAEC_MCP_TOKEN_FILE && -s $OPENAEC_MCP_TOKEN_FILE ]] || { echo 'Cannot read MCP token file' >&2; exit 1; }
  export OPENAEC_MCP='/usr/bin/node /opt/ofs/packaging/docker/mcp-http.mjs'
fi
