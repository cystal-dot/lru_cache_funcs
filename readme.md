# lru_cache_funcs  
Rustとpgrxを使用してPostgreSQLでクエリキャッシュを実現するための拡張機能です。  
lruでキャッシュを管理します。  
型を固定するために返却値はjson固定です。  

### 起動
Dev Containerで開く

下記コマンドでpgrxのinstallとbuild
```
sudo apt-get update && sudo apt-get install -y bison flex
cargo install cargo-pgrx
cargo pgrx init
cargo build
```

```
cargo pgrx run pg14
create extension lru_cache_funcs;
select hello_lru_cache_funcs();
```

### 使い方
キャッシュに載せる
```
lru_cache_funcs=# SELECT execute_with_cache('SELECT 1 AS col1, 2 AS col2, 3 AS col3');
       execute_with_cache       
--------------------------------
 [{"col1":1,"col2":2,"col3":3}]
(1 row)
```

キャッシュをクリアする
```
lru_cache_funcs=# SELECT clear_cache();
     clear_cache      
----------------------
 Query cache cleared.
(1 row)

```

キャッシュサイズを変更する
```
lru_cache_funcs=# select set_cache_size('10');
   set_cache_size    
---------------------
 Cache size updated.
(1 row)
```