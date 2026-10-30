#!/bin/sh
# Fail if the repo contains source documents or local/private paths.
set -eu
bad=0
docs=$(git ls-files | grep -Ei '\.(pdf|epub|djvu|mobi|azw3)$' || true)
if [ -n "$docs" ]; then
  echo "error: source documents must not be committed:"; echo "$docs"; bad=1
fi
if git grep -nIE '/Users/|/home/[a-z]+/|research_papers/|annas-archive|libgen|sci-hub|[0-9a-f]{32}\.(pdf|epub)' -- . ':!tools/check-no-corpus.sh'; then
  echo "error: local paths or corpus-source references found (see above)"; bad=1
fi
exit $bad
