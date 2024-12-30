Dev Containerで開く

下記コマンドでpgrxのinstallとbuild
```
sudo apt-get update && sudo apt-get install -y bison flex
cargo install cargo-pgrx
cargo pgrx init
cargo build
```

起動
```
cargo pgrx run pg14
create extension lru_cache_funcs;
select hello_lru_cache_funcs();
```
