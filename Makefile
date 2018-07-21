.PHONY: update-version

all: update-version

update-version:
	sed -i "s/version:.*$$/version: $(shell cat VERSION)/g" cli/sit-issue.yaml
