// Docker Bake configuration for CreditChain images

variable "REGISTRY" {
  default = "ghcr.io/openibank"
}

variable "TAG" {
  default = "latest"
}

variable "BUILD_PROFILE" {
  default = "maxperf-symbols"
}

variable "FEATURES" {
  default = ""
}

// Git info for vergen (since .git is excluded from Docker context)
variable "VERGEN_GIT_SHA" {
  default = ""
}

variable "VERGEN_GIT_DESCRIBE" {
  default = ""
}

variable "VERGEN_GIT_DIRTY" {
  default = "false"
}

// Common settings for all targets
group "default" {
  targets = ["creditchain"]
}

group "nightly" {
  targets = ["creditchain", "creditchain-profiling"]
}

// Base target with shared configuration
target "_base" {
  dockerfile = "Dockerfile.depot"
  platforms  = ["linux/amd64", "linux/arm64"]
  args = {
    BUILD_PROFILE      = "${BUILD_PROFILE}"
    FEATURES           = "${FEATURES}"
    VERGEN_GIT_SHA     = "${VERGEN_GIT_SHA}"
    VERGEN_GIT_DESCRIBE = "${VERGEN_GIT_DESCRIBE}"
    VERGEN_GIT_DIRTY   = "${VERGEN_GIT_DIRTY}"
  }
  secret = [
    {
      type = "env"
      id   = "DEPOT_TOKEN"
    }
  ]
}
target "_base_profiling" {
  inherits = ["_base"]
  platforms  = ["linux/amd64"]
}

// CreditChain
target "creditchain" {
  inherits = ["_base"]
  args = {
    BINARY        = "creditchaind"
    MANIFEST_PATH = "bin/reth"
  }
  tags = ["${REGISTRY}/creditchain:${TAG}"]
}

target "creditchain-profiling" {
  inherits = ["_base_profiling"]
  args = {
    BINARY        = "creditchaind"
    MANIFEST_PATH = "bin/reth"
    BUILD_PROFILE = "profiling"
    FEATURES      = "jemalloc-prof"
  }
  tags = ["${REGISTRY}/creditchain:nightly-profiling"]
}

// Hive test targets — single-platform, hivetests profile, tar output
target "_base_hive" {
  inherits  = ["_base"]
  platforms = ["linux/amd64"]
  args = {
    BUILD_PROFILE = "hivetests"
  }
}

variable "HIVE_OUTPUT_DIR" {
  default = "./artifacts"
}

target "hive" {
  inherits = ["_base_hive"]
  args = {
    BINARY        = "creditchaind"
    MANIFEST_PATH = "bin/reth"
  }
  tags   = ["creditchain:local"]
  output = ["type=docker,dest=${HIVE_OUTPUT_DIR}/creditchain_image.tar"]
}

// Kurtosis test target
target "kurtosis" {
  inherits  = ["_base_hive"]
  args = {
    BINARY        = "creditchaind"
    MANIFEST_PATH = "bin/reth"
  }
  tags   = ["ghcr.io/openibank/creditchain:kurtosis-ci"]
  output = ["type=docker,dest=${HIVE_OUTPUT_DIR}/creditchain_image.tar"]
}
