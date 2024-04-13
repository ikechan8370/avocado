# avocado bot

基于Kritor协议的bot端，支持多账号。目前仅支持被动Grpc。目前处于初始阶段，开发中。

## 关于插件

支持Rust编写的原生插件，可参考src/service/example/下的示例。

同时支持js脚本，可参考plugins/javascript/example下的示例，可实时修改立刻生效。

> 注意，由于本项目由rust编写，所支持的js脚本不支持node等运行时，仅为脚本运行。目前支持大部分js操作，不支持外部依赖，由rust端提供部分API并由ts对相关API进行类型提示。

> 未来拟打算支持其他脚本语言，如rhai、lua等。

## 安装和使用
todo

## 开发与贡献
todo

## credit
* [kritor](https://github.com/Karin/kritor) 本项目支持的协议，也是本项目实施的动机。
