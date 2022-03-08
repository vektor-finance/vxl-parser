.PHONY: build-js test-js publish-js

# JavaScript

publish-js:
	yarn publish --registry=${NPM_REGISTRY} js/pkg;

build-js:
	wasm-pack build --dev --scope=vektor-finance js; \
	sed -i -e 's/"name": "@vektor-finance\/js"/"name": "@vektor-finance\/vxl-parser"/g' "js/pkg/package.json";

test-js:
	wasm-pack test --node --firefox --chrome --headless js
