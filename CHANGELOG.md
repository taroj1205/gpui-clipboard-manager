# Changelog

## [0.9.0](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.8.0...gpui-clipboard-manager-v0.9.0) (2026-02-20)


### Features

* **img:** allow copying image ([#28](https://github.com/taroj1205/gpui-clipboard-manager/issues/28)) ([5f0b1c4](https://github.com/taroj1205/gpui-clipboard-manager/commit/5f0b1c4ba217f8eb1fc997f265e904ba209f8720))
* **ui:** add context delete for entries ([#26](https://github.com/taroj1205/gpui-clipboard-manager/issues/26)) ([06d55d6](https://github.com/taroj1205/gpui-clipboard-manager/commit/06d55d64638480c065c0c4ef38c7f49533b8704c))


### Bug Fixes

* **release:** update name on the build ([044e0b7](https://github.com/taroj1205/gpui-clipboard-manager/commit/044e0b78e9fa33c4051e85eb0d00e92b40546e9e))

## [0.8.0](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.7.3...gpui-clipboard-manager-v0.8.0) (2026-01-17)


### Features

* add Windows MSI installer support ([#19](https://github.com/taroj1205/gpui-clipboard-manager/issues/19)) ([d7788ef](https://github.com/taroj1205/gpui-clipboard-manager/commit/d7788efd749830729972a3bcd77f24b7095482ca))
* **ci:** add MSI preview workflow ([#24](https://github.com/taroj1205/gpui-clipboard-manager/issues/24)) ([64d2b52](https://github.com/taroj1205/gpui-clipboard-manager/commit/64d2b52c792d4afafd71d3057babf6efbc6c72a2))
* **gpui-component:** add gpui-component ([#17](https://github.com/taroj1205/gpui-clipboard-manager/issues/17)) ([67c2a47](https://github.com/taroj1205/gpui-clipboard-manager/commit/67c2a47bc8d880301acc6ac736cd80cd5e8d272e))


### Bug Fixes

* **ci:** install WiX toolset for cargo wix ([3dda752](https://github.com/taroj1205/gpui-clipboard-manager/commit/3dda75275abeeb2ce505fffb39d02763f180affb))
* correct WiX template for cargo-wix compatibility ([c0ef46a](https://github.com/taroj1205/gpui-clipboard-manager/commit/c0ef46a161d24743e17cbecbd92a72d1235c6f2e))
* **release:** temp revert release workflow changes ([e35deab](https://github.com/taroj1205/gpui-clipboard-manager/commit/e35deabeb8ded24da8ee68cc132456d54e4af81d))

## [0.7.3](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.7.2...gpui-clipboard-manager-v0.7.3) (2026-01-16)


### Bug Fixes

* **subsystem:** update subsystem to windows ([f07d2c7](https://github.com/taroj1205/gpui-clipboard-manager/commit/f07d2c7460f631780ae540ae3d0b7b515a49def1))

## [0.7.2](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.7.1...gpui-clipboard-manager-v0.7.2) (2026-01-16)


### Bug Fixes

* **clippy:** fix errors given by clippy ([#14](https://github.com/taroj1205/gpui-clipboard-manager/issues/14)) ([b4928ad](https://github.com/taroj1205/gpui-clipboard-manager/commit/b4928addd4428c38c66143a8ef6573b292fc55fe))

## [0.7.1](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.7.0...gpui-clipboard-manager-v0.7.1) (2026-01-15)


### Bug Fixes

* **app:** fix terminal opening up on opening ([6dd597e](https://github.com/taroj1205/gpui-clipboard-manager/commit/6dd597ef2c396f5741b5ac082002fce80837e842))

## [0.7.0](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.6.1...gpui-clipboard-manager-v0.7.0) (2026-01-15)


### Features

* **ci:** add ci for checking quality ([#10](https://github.com/taroj1205/gpui-clipboard-manager/issues/10)) ([43b8c7c](https://github.com/taroj1205/gpui-clipboard-manager/commit/43b8c7c28ef027a77ecd05e4fd9e569f5f60526b))
* **click:** add double click to copy feature ([e005787](https://github.com/taroj1205/gpui-clipboard-manager/commit/e0057875d9d3df55d95e96d0603a614aabcbb585))
* **enter:** add press enter to copy the content ([3170940](https://github.com/taroj1205/gpui-clipboard-manager/commit/317094020b691be7f3072a965a3ddef3fc7a5e8d))
* **img:** add image recognition with ocr ([8acfbff](https://github.com/taroj1205/gpui-clipboard-manager/commit/8acfbff1c8a8b55d6b1aec8dc86ba927d3446890))
* **link:** add link summary ([cf8e34f](https://github.com/taroj1205/gpui-clipboard-manager/commit/cf8e34f5371526d5ed84847b7e508fbd8d88552f))
* **link:** add nice ui for link summary in info section ([502dcaa](https://github.com/taroj1205/gpui-clipboard-manager/commit/502dcaacee16adce2b892488acfb9eb3bcaf55fb))
* **link:** allow clicking on the link to open it ([fee7df3](https://github.com/taroj1205/gpui-clipboard-manager/commit/fee7df3965b6a31f41a0a00ecec944143a439d2e))


### Bug Fixes

* **build:** fix build opening terminal ([#8](https://github.com/taroj1205/gpui-clipboard-manager/issues/8)) ([b598125](https://github.com/taroj1205/gpui-clipboard-manager/commit/b598125f3536339c39966b8c2e49a0860bcb67f0))
* **focus:** make it focus input on opening window ([0567c81](https://github.com/taroj1205/gpui-clipboard-manager/commit/0567c8119e10e5b252c3b0f475002d0ecfc3ee5e))
* **popup:** fix crash and trim text ([f8b808a](https://github.com/taroj1205/gpui-clipboard-manager/commit/f8b808aa584a7186a773b12f1dc548a5995eb566))
* **release:** build before publishing a release ([d1aa89b](https://github.com/taroj1205/gpui-clipboard-manager/commit/d1aa89b2a6f3cf639e5575ee68ca35f6bf77b8b0))
* **release:** draft the release and wait for build ([3f6fbdc](https://github.com/taroj1205/gpui-clipboard-manager/commit/3f6fbdc1043ebaae2f6654852570612363129fa1))

## [0.6.1](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.6.0...gpui-clipboard-manager-v0.6.1) (2026-01-15)


### Bug Fixes

* **build:** fix build opening terminal ([#8](https://github.com/taroj1205/gpui-clipboard-manager/issues/8)) ([b598125](https://github.com/taroj1205/gpui-clipboard-manager/commit/b598125f3536339c39966b8c2e49a0860bcb67f0))

## [0.6.0](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.5.0...gpui-clipboard-manager-v0.6.0) (2026-01-10)


### Features

* **click:** add double click to copy feature ([e005787](https://github.com/taroj1205/gpui-clipboard-manager/commit/e0057875d9d3df55d95e96d0603a614aabcbb585))
* **enter:** add press enter to copy the content ([3170940](https://github.com/taroj1205/gpui-clipboard-manager/commit/317094020b691be7f3072a965a3ddef3fc7a5e8d))
* **img:** add image recognition with ocr ([8acfbff](https://github.com/taroj1205/gpui-clipboard-manager/commit/8acfbff1c8a8b55d6b1aec8dc86ba927d3446890))
* **link:** add link summary ([cf8e34f](https://github.com/taroj1205/gpui-clipboard-manager/commit/cf8e34f5371526d5ed84847b7e508fbd8d88552f))
* **link:** add nice ui for link summary in info section ([502dcaa](https://github.com/taroj1205/gpui-clipboard-manager/commit/502dcaacee16adce2b892488acfb9eb3bcaf55fb))
* **link:** allow clicking on the link to open it ([fee7df3](https://github.com/taroj1205/gpui-clipboard-manager/commit/fee7df3965b6a31f41a0a00ecec944143a439d2e))


### Bug Fixes

* **focus:** make it focus input on opening window ([0567c81](https://github.com/taroj1205/gpui-clipboard-manager/commit/0567c8119e10e5b252c3b0f475002d0ecfc3ee5e))
* **popup:** fix crash and trim text ([f8b808a](https://github.com/taroj1205/gpui-clipboard-manager/commit/f8b808aa584a7186a773b12f1dc548a5995eb566))
* **release:** build before publishing a release ([d1aa89b](https://github.com/taroj1205/gpui-clipboard-manager/commit/d1aa89b2a6f3cf639e5575ee68ca35f6bf77b8b0))
* **release:** draft the release and wait for build ([3f6fbdc](https://github.com/taroj1205/gpui-clipboard-manager/commit/3f6fbdc1043ebaae2f6654852570612363129fa1))

## [0.5.0](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.4.1...gpui-clipboard-manager-v0.5.0) (2026-01-10)


### Features

* **click:** add double click to copy feature ([e005787](https://github.com/taroj1205/gpui-clipboard-manager/commit/e0057875d9d3df55d95e96d0603a614aabcbb585))
* **enter:** add press enter to copy the content ([3170940](https://github.com/taroj1205/gpui-clipboard-manager/commit/317094020b691be7f3072a965a3ddef3fc7a5e8d))
* **img:** add image recognition with ocr ([8acfbff](https://github.com/taroj1205/gpui-clipboard-manager/commit/8acfbff1c8a8b55d6b1aec8dc86ba927d3446890))
* **link:** add link summary ([cf8e34f](https://github.com/taroj1205/gpui-clipboard-manager/commit/cf8e34f5371526d5ed84847b7e508fbd8d88552f))
* **link:** add nice ui for link summary in info section ([502dcaa](https://github.com/taroj1205/gpui-clipboard-manager/commit/502dcaacee16adce2b892488acfb9eb3bcaf55fb))
* **link:** allow clicking on the link to open it ([fee7df3](https://github.com/taroj1205/gpui-clipboard-manager/commit/fee7df3965b6a31f41a0a00ecec944143a439d2e))


### Bug Fixes

* **focus:** make it focus input on opening window ([0567c81](https://github.com/taroj1205/gpui-clipboard-manager/commit/0567c8119e10e5b252c3b0f475002d0ecfc3ee5e))
* **popup:** fix crash and trim text ([f8b808a](https://github.com/taroj1205/gpui-clipboard-manager/commit/f8b808aa584a7186a773b12f1dc548a5995eb566))
* **release:** build before publishing a release ([d1aa89b](https://github.com/taroj1205/gpui-clipboard-manager/commit/d1aa89b2a6f3cf639e5575ee68ca35f6bf77b8b0))
* **release:** draft the release and wait for build ([3f6fbdc](https://github.com/taroj1205/gpui-clipboard-manager/commit/3f6fbdc1043ebaae2f6654852570612363129fa1))

## [0.4.1](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.4.0...gpui-clipboard-manager-v0.4.1) (2026-01-10)


### Bug Fixes

* **focus:** make it focus input on opening window ([0567c81](https://github.com/taroj1205/gpui-clipboard-manager/commit/0567c8119e10e5b252c3b0f475002d0ecfc3ee5e))
* **release:** draft the release and wait for build ([3f6fbdc](https://github.com/taroj1205/gpui-clipboard-manager/commit/3f6fbdc1043ebaae2f6654852570612363129fa1))

## [0.4.0](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.3.0...gpui-clipboard-manager-v0.4.0) (2026-01-10)


### Features

* **link:** allow clicking on the link to open it ([fee7df3](https://github.com/taroj1205/gpui-clipboard-manager/commit/fee7df3965b6a31f41a0a00ecec944143a439d2e))

## [0.3.0](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.2.0...gpui-clipboard-manager-v0.3.0) (2026-01-10)


### Features

* **enter:** add press enter to copy the content ([3170940](https://github.com/taroj1205/gpui-clipboard-manager/commit/317094020b691be7f3072a965a3ddef3fc7a5e8d))


### Bug Fixes

* **release:** build before publishing a release ([d1aa89b](https://github.com/taroj1205/gpui-clipboard-manager/commit/d1aa89b2a6f3cf639e5575ee68ca35f6bf77b8b0))

## [0.2.0](https://github.com/taroj1205/gpui-clipboard-manager/compare/gpui-clipboard-manager-v0.1.0...gpui-clipboard-manager-v0.2.0) (2026-01-10)


### Features

* **img:** add image recognition with ocr ([8acfbff](https://github.com/taroj1205/gpui-clipboard-manager/commit/8acfbff1c8a8b55d6b1aec8dc86ba927d3446890))
* **link:** add link summary ([cf8e34f](https://github.com/taroj1205/gpui-clipboard-manager/commit/cf8e34f5371526d5ed84847b7e508fbd8d88552f))
* **link:** add nice ui for link summary in info section ([502dcaa](https://github.com/taroj1205/gpui-clipboard-manager/commit/502dcaacee16adce2b892488acfb9eb3bcaf55fb))


### Bug Fixes

* **popup:** fix crash and trim text ([f8b808a](https://github.com/taroj1205/gpui-clipboard-manager/commit/f8b808aa584a7186a773b12f1dc548a5995eb566))
