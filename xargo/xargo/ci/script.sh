set -euxo pipefail

beginswith() { case $2 in "$1"*) true;; *) false;; esac; }

main() {
    cross build --target $TARGET
    cross run --target $TARGET -- -V

    if beginswith nightly $TRAVIS_RUST_VERSION; then
        cargo test --features dev --target $TARGET
        cargo test --target $TARGET
    fi
}

if [ -z $TRAVIS_TAG ] && [ $TRAVIS_BRANCH != master ] || [ $TRAVIS_EVENT_TYPE = cron ]; then
    main
fi
