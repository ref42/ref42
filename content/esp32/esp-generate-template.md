---
title: '配置ESP32模板工程，此后的所有工程均可按此流程配置'
description: '使用 esp-generate 生成 ESP32-C3 Rust 初始工程，并配置 espflash、日志、调试和 VSCode 集成。'
weight: 10
---

## 使用 esp-generate 生成一个初始工程

> 进行此实验前，务必先查看[基础环境搭建](/toolchain/rust_dev_base_env/)，完成之后，继续完成[ESP32 Rust环境搭建](/toolchain/esp32-dev-setup/)

完成以上步骤之后,来到合适的文件目录下，最好是新建一个目录用于存放工程，在此目录下打开终端，输入以下命令，并按下回车（`Enter`）。也可以先输入`esp-generate`，按下回车（`Enter`）之后，再根据提示一步一步选择。

```bash
esp-generate --chip=esp32c3 esp32c3_embassy_dht11_demo
```
稍微解释一下，上面这行命令的作用

```bash
esp-generate # 调用该工具
--chip=esp32c3  # 传入参数，指定芯片具体名称
esp32c3_embassy_dht11_demo  #   待生成工程的工程名，推荐使用snake_case（所有单词小写，以下划线分割单词）命名，这是一个传统，也是一个约定，否则可能会有警告
```

此时，命令行会打开一个类似`GUI`的界面，使用键盘上的"上下左右"箭头来选择相应的`feature`，高亮部分即代表待选中的`feature`，按下回车（`Enter`）则会选中此项`feature`，并且在此`feature`前显示✅

![esp-generate feature selection screen](/esp32/template-project/esp-generate-00.png)

`feature`前带有[▶]符号，表示该`feature`含有多个子`feature`，使用箭头选中高亮，并且按下回车（`Enter`）后，会打开新的界面。

![esp-generate feature group screen](/esp32/template-project/esp-generate-01.png)

此时，高亮区域选中了`▶  Flashing, logging and debugging (espflash)`，按下回车（`Enter`），来到新的界面。

![espflash feature group in esp-generate](/esp32/template-project/esp-generate-02.png)

选中下图的`feature`，按照从上至下排序，当前选中了`2`和`3`。

![selected flashing and logging features in esp-generate](/esp32/template-project/esp-generate-03.png)

注意：`1`和`2`只能二选一，工程里编写代码时也只能二选一，否则会报错。

完成选定后，按下`ESC`，返回主界面，继续开启`feature`。

此时，高亮区域选中了`▶  Optional editor integration`，按下回车（`Enter`），来到新的界面。

![optional editor integration menu in esp-generate](/esp32/template-project/esp-generate-04.png)

这个界面主要是选择你将要使用的编辑器，选定之后，可以自动配置工程适应编辑器，打开后不会报错，并且会推荐安装部分插件。

![VSCode editor integration selected in esp-generate](/esp32/template-project/esp-generate-05.png)

选定上图`feature`后，继续按下`ESC`，回到主界面。

此时，我们已经选定了所有`feature`，按下`S`即`Save`，即可保存并且生成工程。终端里也会有提示信息。

![esp-generate project creation complete](/esp32/template-project/esp-generate-06.png)

```bash
🆗 Rust (stable): 1.89.0
🆗 espflash: 4.0.1
🆗 probe-rs: 0.29.1
```

至此，创建工程模板就已经完成了，之后所有的工程模板都是一样的操作流程，不同时期配置的工具链，以上工具版本信息不一致是正常现象。
