# Rime 命令行工具

集成一部分 librime 的提供的工具，如數據部署工具、生成配置補丁等。
遠景是與 rime-make 配合，實現全功能的配方管理器，
幫助輸入方案設計者及用家以配方爲單位創作、分發、使用 Rime 輸入法的數據。
可用於命令行、持續集成環境中 Rime 方案及配置的編譯、離線部署。
也可當作以配方爲核心的圖形配置程序的後端。

## 構建腳本

``` shell
cargo make
```

## 運行程序

``` shell
cargo run -- <子命令> <參數...>
```
