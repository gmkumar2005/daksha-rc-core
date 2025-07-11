# Changelog

## [2.1.0](https://github.com/gmkumar2005/daksha-rc-core/compare/rc-web-v2.0.0...rc-web-v2.1.0) (2025-07-11)


### Features

* removed duplicate runs ([42bbc75](https://github.com/gmkumar2005/daksha-rc-core/commit/42bbc753e6bc5d1f32717ced5be065486b913165))

## [2.0.0](https://github.com/gmkumar2005/daksha-rc-core/compare/rc-web-v1.0.1...rc-web-v2.0.0) (2025-07-11)


### ⚠ BREAKING CHANGES

* redesign release-please and build-simple
* The image API has changed completely. Old image format configurations will no longer work and must be updated.

### Features

* add caching to database health checks ([49723b0](https://github.com/gmkumar2005/daksha-rc-core/commit/49723b08bb25933d2e35e8018fc8dab4324bac8a))
* API responses ([dfaf27c](https://github.com/gmkumar2005/daksha-rc-core/commit/dfaf27cb8c113bee1aaadea879cdacea5518943a))
* Create entity_projections when activated ([4c3cafa](https://github.com/gmkumar2005/daksha-rc-core/commit/4c3cafad349c08f655cf128cc2f0f84b564b3fc2))
* Database connection scripts ([e80f547](https://github.com/gmkumar2005/daksha-rc-core/commit/e80f54768c0b4056ca07a0423c3524faa675b89e))
* definitions table enhancements ([c2a6823](https://github.com/gmkumar2005/daksha-rc-core/commit/c2a6823308ffdb76c8e1c49e1fbfc92d2c2f663e))
* disabled integration tests ([959fd07](https://github.com/gmkumar2005/daksha-rc-core/commit/959fd07cc317736fc5ee38ae6ab22d161b1807c4))
* Documentation updates ([af1e176](https://github.com/gmkumar2005/daksha-rc-core/commit/af1e176b42fe90fc5fb1e184e0d32237cc334426))
* dropped docker-compose ([020ab4e](https://github.com/gmkumar2005/daksha-rc-core/commit/020ab4ecd0bbc8ccbb5789c5359ea566c56e7d5c))
* implement caching functionality ([3f76116](https://github.com/gmkumar2005/daksha-rc-core/commit/3f7611637c9b84988a003e8c87be5fd668b12f62))
* optimized rust ci [#2](https://github.com/gmkumar2005/daksha-rc-core/issues/2) ([4d2af76](https://github.com/gmkumar2005/daksha-rc-core/commit/4d2af762c03a2bcaf09815860d36b0b6cdb479a4))
* optimized rust ci [#3](https://github.com/gmkumar2005/daksha-rc-core/issues/3) ([f1a7c22](https://github.com/gmkumar2005/daksha-rc-core/commit/f1a7c22aa659000d9bd73aacac12a035fcdd4422))
* optimized rust ci code ftm [#4](https://github.com/gmkumar2005/daksha-rc-core/issues/4) ([a431533](https://github.com/gmkumar2005/daksha-rc-core/commit/a43153372bbcf5b60e0932164ee10e123e01e4f0))
* redesign image processing pipeline ([7fd78af](https://github.com/gmkumar2005/daksha-rc-core/commit/7fd78af5cbc2e01412b6019c002f69fcc2589223))
* redesign release-please and build-simple ([d3d52d5](https://github.com/gmkumar2005/daksha-rc-core/commit/d3d52d5195deacab13548d7933c2103acc3f292d))


### Bug Fixes

* 123: sqlx migrations run at startup, os_id introduced ([a447a89](https://github.com/gmkumar2005/daksha-rc-core/commit/a447a8990cfa0795bf9858da9c46f4e11d4c1ac1))
* code ftm removed dead code [#6](https://github.com/gmkumar2005/daksha-rc-core/issues/6) ([e343f3c](https://github.com/gmkumar2005/daksha-rc-core/commit/e343f3c541fd8ce7d09589d86caafc0d2d4795fb))
* optimized rust ci code ftm [#5](https://github.com/gmkumar2005/daksha-rc-core/issues/5) ([7d158eb](https://github.com/gmkumar2005/daksha-rc-core/commit/7d158eb862f4b9586da94d08b45b89313d000aa5))

## [1.0.0](https://github.com/gmkumar2005/daksha-rc-core/compare/rc-web-v0.4.0...rc-web-v1.0.0) (2025-07-11)


### ⚠ BREAKING CHANGES

* The image API has changed completely. Old image format configurations will no longer work and must be updated.

### Features

* redesign image processing pipeline ([7fd78af](https://github.com/gmkumar2005/daksha-rc-core/commit/7fd78af5cbc2e01412b6019c002f69fcc2589223))

## [0.4.0](https://github.com/gmkumar2005/daksha-rc-core/compare/rc-web-v0.3.0...rc-web-v0.4.0) (2025-07-11)


### Features

* disabled integration tests ([959fd07](https://github.com/gmkumar2005/daksha-rc-core/commit/959fd07cc317736fc5ee38ae6ab22d161b1807c4))
* optimized rust ci [#2](https://github.com/gmkumar2005/daksha-rc-core/issues/2) ([4d2af76](https://github.com/gmkumar2005/daksha-rc-core/commit/4d2af762c03a2bcaf09815860d36b0b6cdb479a4))
* optimized rust ci [#3](https://github.com/gmkumar2005/daksha-rc-core/issues/3) ([f1a7c22](https://github.com/gmkumar2005/daksha-rc-core/commit/f1a7c22aa659000d9bd73aacac12a035fcdd4422))
* optimized rust ci code ftm [#4](https://github.com/gmkumar2005/daksha-rc-core/issues/4) ([a431533](https://github.com/gmkumar2005/daksha-rc-core/commit/a43153372bbcf5b60e0932164ee10e123e01e4f0))


### Bug Fixes

* code ftm removed dead code [#6](https://github.com/gmkumar2005/daksha-rc-core/issues/6) ([e343f3c](https://github.com/gmkumar2005/daksha-rc-core/commit/e343f3c541fd8ce7d09589d86caafc0d2d4795fb))
* optimized rust ci code ftm [#5](https://github.com/gmkumar2005/daksha-rc-core/issues/5) ([7d158eb](https://github.com/gmkumar2005/daksha-rc-core/commit/7d158eb862f4b9586da94d08b45b89313d000aa5))

## [0.3.0](https://github.com/gmkumar2005/daksha-rc-core/compare/rc-web-v0.2.0...rc-web-v0.3.0) (2025-07-11)


### Features

* dropped docker-compose ([020ab4e](https://github.com/gmkumar2005/daksha-rc-core/commit/020ab4ecd0bbc8ccbb5789c5359ea566c56e7d5c))

## [0.2.0](https://github.com/gmkumar2005/daksha-rc-core/compare/rc-web-v0.1.10...rc-web-v0.2.0) (2025-07-11)


### Features

* add caching to database health checks ([49723b0](https://github.com/gmkumar2005/daksha-rc-core/commit/49723b08bb25933d2e35e8018fc8dab4324bac8a))
* API responses ([dfaf27c](https://github.com/gmkumar2005/daksha-rc-core/commit/dfaf27cb8c113bee1aaadea879cdacea5518943a))
* Create entity_projections when activated ([4c3cafa](https://github.com/gmkumar2005/daksha-rc-core/commit/4c3cafad349c08f655cf128cc2f0f84b564b3fc2))
* Database connection scripts ([e80f547](https://github.com/gmkumar2005/daksha-rc-core/commit/e80f54768c0b4056ca07a0423c3524faa675b89e))
* definitions table enhancements ([c2a6823](https://github.com/gmkumar2005/daksha-rc-core/commit/c2a6823308ffdb76c8e1c49e1fbfc92d2c2f663e))
* Documentation updates ([af1e176](https://github.com/gmkumar2005/daksha-rc-core/commit/af1e176b42fe90fc5fb1e184e0d32237cc334426))
* implement caching functionality ([3f76116](https://github.com/gmkumar2005/daksha-rc-core/commit/3f7611637c9b84988a003e8c87be5fd668b12f62))


### Bug Fixes

* 123: sqlx migrations run at startup, os_id introduced ([a447a89](https://github.com/gmkumar2005/daksha-rc-core/commit/a447a8990cfa0795bf9858da9c46f4e11d4c1ac1))
