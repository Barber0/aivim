# Aivim P0/P1 未实现功能教学

本文档介绍 Aivim 中尚未实现的 P0（核心）和 P1（基础）功能，帮助您提前了解这些功能的具体用途和使用方法。

---

## 目录

1. [宏录制 (Macro Recording)](#1-宏录制-macro-recording)
2. [多点撤销/撤销树 (Undo Tree)](#2-多点撤销撤销树-undo-tree)
3. [高级搜索与替换](#3-高级搜索与替换)
4. [标记系统 (Mark System)](#4-标记系统-mark-system)
5. [跳转列表 (Jump List)](#5-跳转列表-jump-list)
6. [行号显示](#6-行号显示)
7. [配置系统](#7-配置系统)
8. [键位映射](#8-键位映射)
9. [外部命令](#9-外部命令)

---

## 1. 宏录制 (Macro Recording)

**优先级**: P1  
**Vim 命令**: `q{register}` 录制, `@{register}` 回放

### 什么是宏？

宏（Macro）是一种将一系列键盘操作录制下来，然后重复回放的功能。它是 Vim 中实现重复操作自动化的核心功能。

### 使用场景

**场景 1：批量格式化数据**

假设您有一个 CSV 文件，需要将每行格式化为特定格式：
```
原始数据：
Alice,25,Engineer
Bob,30,Designer
Charlie,35,Manager

目标格式：
Name: Alice, Age: 25, Job: Engineer
Name: Bob, Age: 30, Job: Designer
Name: Charlie, Age: 35, Job: Manager
```

**操作步骤**：
```vim
" 1. 将光标移到第一行
" 2. 开始录制宏到寄存器 a
qa

" 3. 执行格式化操作
^                    " 跳到行首
iName: <Esc>         " 插入 "Name: "
f,i, Age: <Esc>      " 找到逗号，插入 ", Age: "
f,i, Job: <Esc>      " 找到逗号，插入 ", Job: "
j                    " 移到下一行

" 4. 停止录制
q

" 5. 回放宏处理剩余行
2@a                  " 回放 2 次（处理第 2、3 行）
```

**场景 2：批量重命名变量**

```vim
" 录制宏：将 old_var 替换为 new_var，然后跳到下一个出现位置
qa
*                    " 搜索当前单词
cwnew_var<Esc>       " 替换为新变量名
q

" 回放 10 次
10@a
```

### 相关命令

| 命令 | 功能 |
|------|------|
| `q{a-z}` | 开始录制宏到指定寄存器 |
| `q` | 停止录制 |
| `@{a-z}` | 回放指定寄存器的宏 |
| `@@` | 回放上一次的宏 |
| `{n}@{a-z}` | 回放 n 次 |
| `:reg {a-z}` | 查看宏内容 |

### 进阶技巧

**递归宏**：宏可以调用自己实现循环
```vim
" 录制一个处理一行并调用自己的宏
qa
...操作...
@a                   " 宏内部调用自己
q

" 开始递归（会自动停止当操作失败时）
@a
```

---

## 2. 多点撤销/撤销树 (Undo Tree)

**优先级**: P1  
**Vim 命令**: `u` 撤销, `C-r` 重做, `:undolist` 查看分支

### 什么是撤销树？

普通编辑器只有线性的撤销历史（A → B → C → D）。但 Vim 的撤销树允许分支：

```
编辑 A
  └── 编辑 B
        ├── 编辑 C (分支 1)
        │     └── 编辑 D
        └── 撤销后重新编辑 E (分支 2)
              └── 编辑 F
```

### 使用场景

**场景：探索不同实现方案**

您在写函数，尝试了方案 A，不满意撤销后又写了方案 B：

```vim
" 原始代码
function calculate(x) {
    return x * 2;
}

" 方案 A：使用加法
function calculate(x) {
    return x + x;        " 编辑 A
}

" 撤销回原始代码
u

" 方案 B：使用位运算
function calculate(x) {
    return x << 1;       " 编辑 B（创建新分支）
}
```

现在您可以在两个方案间切换：
```vim
:undolist             " 查看所有分支
g-                    " 回到更早的状态（在分支间移动）
g+                    " 回到更新的状态
```

### 相关命令

| 命令 | 功能 |
|------|------|
| `u` | 撤销 |
| `C-r` | 重做 |
| `g-` | 回到更早的撤销状态（时间旅行） |
| `g+` | 回到更新的撤销状态 |
| `:undolist` | 显示撤销树 |
| `:undo {n}` | 跳到第 n 个撤销状态 |

### 可视化

一些 Vim 插件（如 undotree）可以图形化显示撤销树：

```
Undo Tree
═══════════════════════════════════════
  3     [原始代码]
  │
  2     [方案 A] ◄── 当前位置
  │
  1     [方案 B]
  │
  0     [方案 C]
═══════════════════════════════════════
```

---

## 3. 高级搜索与替换

**优先级**: P2  
**Vim 命令**: `:s`, `:g`, `:v`

### 3.1 替换命令 (:s)

**基本语法**：
```vim
:[range]s/old/new/[flags]
```

**使用场景 1：行内替换**

```vim
" 将当前行的第一个 foo 替换为 bar
:s/foo/bar/

" 将当前行的所有 foo 替换为 bar (g = global)
:s/foo/bar/g

" 替换第 5-10 行的所有 foo
:5,10s/foo/bar/g

" 替换整个文件的所有 foo
:%s/foo/bar/g

" 替换时确认 (c = confirm)
:%s/foo/bar/gc
" 会提示：replace with bar (y/n/a/q/l/^E/^Y)?
```

**使用场景 2：使用正则表达式**

```vim
" 删除行尾空格
:%s/\s\+$//g

" 将多个空行合并为一个
:%s/^\n\+/\r/g

" 交换两个单词的位置（foo bar -> bar foo）
:%s/\(foo\) \(bar\)/\2 \1/g

" 给每行添加行号
:%s/^/\=line('.').'. '/g
```

**使用场景 3：替换中的特殊符号**

```vim
" 使用当前搜索模式
:%s//replacement/g

" 使用上一次的替换模式
:%s~/new~/g

" 使用寄存器内容替换
:%s/foo/\=@a/g        " 用寄存器 a 的内容替换 foo
```

### 3.2 全局命令 (:g)

**语法**：
```vim
:[range]g/pattern/command
```

**使用场景 1：删除匹配行**

```vim
" 删除所有包含 "TODO" 的行
:g/TODO/d

" 删除所有空行
:g/^$/d

" 删除所有以 # 开头的注释行（Python）
:g/^#/d
```

**使用场景 2：对匹配行执行操作**

```vim
" 对所有包含 "ERROR" 的行执行替换
:g/ERROR/s/foo/bar/g

" 复制所有包含 "IMPORTANT" 的行到文件末尾
:g/IMPORTANT/t$

" 收集所有函数定义到文件末尾
:g/^func/t$
```

**使用场景 3：反转行顺序**

```vim
" 将文件行顺序反转
:g/^/m0
```

### 3.3 反向全局命令 (:v)

`:v` 是 `:g` 的反向操作，对**不匹配**的行执行命令：

```vim
" 删除所有不包含 "keep" 的行
:v/keep/d

" 只保留包含数字的行
:v/\d/d
```

### 命令速查表

| 命令 | 功能 |
|------|------|
| `:s/old/new/` | 当前行首次替换 |
| `:s/old/new/g` | 当前行全部替换 |
| `:%s/old/new/g` | 全文替换 |
| `:%s/old/new/gc` | 全文替换（确认） |
| `:5,10s/old/new/g` | 5-10行替换 |
| `:'<,'>s/old/new/g` | 选中区域替换 |
| `:g/pattern/d` | 删除匹配行 |
| `:g/pattern/s/old/new/g` | 对匹配行替换 |
| `:v/pattern/d` | 删除不匹配行 |

---

## 4. 标记系统 (Mark System)

**优先级**: P2  
**Vim 命令**: `m{mark}`, `'{mark}`, `` `{mark} ``

### 什么是标记？

标记是在文件中设置的书签，可以快速跳转到特定位置。Vim 有两类标记：

- **小写标记 (a-z)**：单文件内跳转
- **大写标记 (A-Z)**：跨文件全局跳转

### 使用场景

**场景 1：在函数间快速跳转**

```vim
" 在 main 函数处设置标记 m
mm

" 在辅助函数处设置标记 h
function helper() {
    ...
}                    " 光标在这里，按 mh

" 在工具函数处设置标记 u
function utility() {
    ...
}                    " 光标在这里，按 mu

" 现在可以快速跳转
'm                   " 跳转到 main 函数
'h                   " 跳转到 helper 函数
'u                   " 跳转到 utility 函数
```

**场景 2：跨文件跳转**

```vim
" 在 config.h 中设置全局标记 C
mC

" 在 main.c 中设置全局标记 M
mM

" 跳转到 config.h 的标记位置
'C

" 跳回 main.c
'M
```

**场景 3：选择标记之间的文本**

```vim
" 设置标记 a
ma

" 移动到其他位置

" 选择从标记 a 到当前位置
v'a                  " 字符选择
V'a                  " 行选择
```

### 特殊标记

Vim 自动维护一些特殊标记：

| 标记 | 含义 |
|------|------|
| `` ` `` | 上次跳转前的位置 |
| `.` | 上次修改的位置 |
| `^` | 上次插入模式的位置 |
| `[` | 上次修改/复制的起始位置 |
| `]` | 上次修改/复制的结束位置 |
| `<` | 上次可视选择的起始位置 |
| `>` | 上次可视选择的结束位置 |

**使用示例**：
```vim
" 跳转到上次修改的位置并居中显示
`.

" 跳转到上次插入的位置并开始插入
gi

" 选择上次修改的文本
`[v`]
```

### 标记列表

```vim
:marks                " 显示所有标记
:marks aAbB           " 显示指定标记
```

---

## 5. 跳转列表 (Jump List)

**优先级**: P2  
**Vim 命令**: `C-o` 后退, `C-i` 前进

### 什么是跳转列表？

跳转列表记录了您在文件中的跳转历史，包括：
- 搜索跳转 (/，?，n，N)
- 行跳转 (G，gg)
- 标记跳转 ('a，`a)
- 文件跳转 (gf，:e)

### 使用场景

**场景：代码追踪**

```vim
" 1. 在 main 函数中
" 2. 搜索函数调用
/calculate

" 3. 跳转到函数定义
gd

" 4. 查看函数内部...

" 5. 跳回 main 函数
C-o                   " 跳回上一个位置

" 6. 再次前进
c-i                   " 跳转到下一个位置
```

**场景：多文件导航**

```vim
" 1. 在 file1.c 中
" 2. 跳转到 file2.c 中的定义
gd

" 3. 查看后又跳转到 file3.c
gd

" 4. 快速返回
C-o                   " 回到 file2.c
C-o                   " 回到 file1.c

" 5. 再次前进
c-i                   " 到 file2.c
c-i                   " 到 file3.c
```

### 跳转列表命令

| 命令 | 功能 |
|------|------|
| `C-o` | 跳转到列表中的上一个位置 |
| `C-i` 或 `Tab` | 跳转到列表中的下一个位置 |
| `:jumps` | 查看跳转列表 |
| `:clearjumps` | 清空跳转列表 |

### 跳转列表示例

```vim
:jumps
 jump  line  col  file/text
   4    42    5   main.c
   3   156   12   utils.c
   2    89    0   config.h
   1    10    0   README.md
>                                 " > 表示当前位置
```

---

## 6. 行号显示

**优先级**: P1  
**Vim 命令**: `:set number`, `:set relativenumber`

### 行号类型

**绝对行号**：
```vim
:set number           " 或 :set nu
```
显示效果：
```
  1 #include <stdio.h>
  2 
  3 int main() {
  4     printf("Hello\n");
  5     return 0;
  6 }
```

**相对行号**：
```vim
:set relativenumber   " 或 :set rnu
```
显示效果：
```
  3 #include <stdio.h>
  2 
  1 int main() {
  0     printf("Hello\n");     " 当前行显示 0
  1     return 0;
  2 }
```

**混合行号**（当前行绝对，其他相对）：
```vim
:set number relativenumber
```
显示效果：
```
  3 #include <stdio.h>
  2 
  1 int main() {
4     printf("Hello\n");       " 当前行显示绝对行号 4
  1     return 0;
  2 }
```

### 使用场景

**场景 1：相对行号 + 操作**

```vim
:set relativenumber

" 当前在第 10 行，想删除下面 5 行
5dd                   " 直接看到是 5 行，不需要计算

" 复制下面 3 行
3yy

" 跳转到上面 8 行
8k
```

**场景 2：绝对行号 + 跳转**

```vim
:set number

" 直接跳转到第 150 行
150G

" 在命令中指定行号
:150
```

### 相关命令

| 命令 | 功能 |
|------|------|
| `:set nu` | 启用绝对行号 |
| `:set nonu` | 关闭绝对行号 |
| `:set rnu` | 启用相对行号 |
| `:set nornu` | 关闭相对行号 |
| `:set nu rnu` | 混合行号 |
| `:set nonu nornu` | 关闭所有行号 |

---

## 7. 配置系统

**优先级**: P1  
**Vim 命令**: `:set`, `:let`, 配置文件

### 基础配置

**布尔选项**（开/关）：
```vim
:set number           " 开启行号
:set nonumber         " 关闭行号
:set number!          " 切换行号状态
set number?          " 查询当前值
```

**数值选项**：
```vim
:set tabstop=4        " Tab 宽度为 4
:set shiftwidth=4     " 缩进宽度为 4
:set textwidth=80     " 自动换行宽度
```

**字符串选项**：
```vim
:set filetype=python  " 设置文件类型
:set encoding=utf-8   " 设置编码
```

### 常用配置项

| 选项 | 说明 | 常用值 |
|------|------|--------|
| `number` | 显示行号 | `on`, `off` |
| `relativenumber` | 相对行号 | `on`, `off` |
| `tabstop` | Tab 字符宽度 | `2`, `4`, `8` |
| `shiftwidth` | 自动缩进宽度 | `2`, `4`, `8` |
| `expandtab` | Tab 转换为空格 | `on`, `off` |
| `autoindent` | 自动缩进 | `on`, `off` |
| `smartindent` | 智能缩进 | `on`, `off` |
| `wrap` | 自动换行 | `on`, `off` |
| `hlsearch` | 高亮搜索结果 | `on`, `off` |
| `incsearch` | 增量搜索 | `on`, `off` |
| `ignorecase` | 搜索忽略大小写 | `on`, `off` |
| `smartcase` | 智能大小写 | `on`, `off` |
| `cursorline` | 高亮当前行 | `on`, `off` |
| `cursorcolumn` | 高亮当前列 | `on`, `off` |
| `colorcolumn` | 标记列 | `80`, `120` |
| `laststatus` | 状态栏显示 | `0`, `1`, `2` |
| `ruler` | 显示光标位置 | `on`, `off` |
| `showcmd` | 显示部分命令 | `on`, `off` |
| `showmode` | 显示当前模式 | `on`, `off` |
| `undofile` | 持久化撤销历史 | `on`, `off` |
| `backup` | 创建备份文件 | `on`, `off` |
| `swapfile` | 创建交换文件 | `on`, `off` |

### 配置文件

Vim 使用 `.vimrc` 作为配置文件，Aivim 可能使用类似的配置：

```vim
" ~/.aivimrc 或 ~/.config/aivim/config

" 基础设置
set number              " 显示行号
set relativenumber      " 相对行号
set tabstop=4           " Tab 宽度
set shiftwidth=4        " 缩进宽度
set expandtab           " Tab 转空格
set autoindent          " 自动缩进
set smartindent         " 智能缩进

" 搜索设置
set hlsearch            " 高亮搜索
set incsearch           " 增量搜索
set ignorecase          " 忽略大小写
set smartcase           " 智能大小写

" 显示设置
set cursorline          " 高亮当前行
set colorcolumn=80      " 标记第 80 列
set laststatus=2        " 总是显示状态栏
set ruler               " 显示光标位置
set showcmd             " 显示部分命令
set showmode            " 显示当前模式

" 文件设置
set encoding=utf-8      " UTF-8 编码
set fileformat=unix     " Unix 换行符
set undofile            " 持久化撤销
set backup              " 创建备份
set swapfile            " 创建交换文件

" 行为设置
set backspace=indent,eol,start  " 退格键行为
set clipboard=unnamed   " 使用系统剪贴板
set mouse=a             " 启用鼠标
set hidden              " 允许隐藏未保存的缓冲区
```

---

## 8. 键位映射

**优先级**: P1  
**Vim 命令**: `:map`, `:nmap`, `:imap`, `:vmap`

### 映射类型

| 命令 | 模式 | 说明 |
|------|------|------|
| `:map` | Normal + Visual | 通用映射 |
| `:nmap` | Normal | 普通模式映射 |
| `:imap` | Insert | 插入模式映射 |
| `:vmap` | Visual | 可视模式映射 |
| `:cmap` | Command | 命令模式映射 |
| `:tmap` | Terminal | 终端模式映射 |

### 使用场景

**场景 1：简化常用操作**

```vim
" 使用 ; 代替 :（少按 Shift）
nmap ; :

" 快速保存
nmap <Space>w :w<CR>

" 快速退出
nmap <Space>q :q<CR>

" 快速保存并退出
nmap <Space>x :x<CR>
```

**场景 2：窗口导航简化**

```vim
" 使用 Ctrl + 方向键导航窗口
nmap <C-h> <C-w>h
nmap <C-j> <C-w>j
nmap <C-k> <C-w>k
nmap <C-l> <C-w>l
```

**场景 3：快速编辑**

```vim
" 在插入模式使用 jj 退出到 Normal 模式
imap jj <Esc>

" 在 Normal 模式快速进入插入模式
nmap <Space>i i
nmap <Space>a a

" 快速在行尾添加分号
nmap <Space>; $a;<Esc>
```

**场景 4：Leader 键映射**

```vim
" 设置 Leader 键
let mapleader = " "

" 使用 Leader 的映射
nmap <Leader>w :w<CR>
nmap <Leader>q :q<CR>
nmap <Leader>e :e 
nmap <Leader>n :new<CR>
nmap <Leader>l :ls<CR>
nmap <Leader>b :b 
nmap <Leader>d :bd<CR>
```

### 特殊键表示

| 表示 | 含义 |
|------|------|
| `<CR>` | 回车键 |
| `<Esc>` | Esc 键 |
| `<Space>` | 空格键 |
| `<Tab>` | Tab 键 |
| `<BS>` | 退格键 |
| `<C-x>` | Ctrl + x |
| `<A-x>` 或 `<M-x>` | Alt + x |
| `<F1>` - `<F12>` | 功能键 |
| `<Leader>` | Leader 键 |
| `<LocalLeader>` | 本地 Leader 键 |
| `<SID>` | 脚本 ID |
| `<Plug>` | 插件映射前缀 |

### 查看和删除映射

```vim
:map                  " 查看所有映射
:nmap                 " 查看 Normal 模式映射
:imap                 " 查看 Insert 模式映射

:unmap <key>          " 删除映射
:nunmap <key>         " 删除 Normal 模式映射
:mapclear             " 清除所有映射
```

---

## 9. 外部命令

**优先级**: P1  
**Vim 命令**: `:!`, `:r !`, `:w !`

### 执行外部命令

**基本语法**：
```vim
:!command             " 执行命令并显示输出
```

### 使用场景

**场景 1：编译运行**

```vim
" 编译当前文件
:!gcc % -o program

" 运行程序
:!./program

" 编译并运行（组合）
:!gcc % -o program && ./program

" Python 脚本
:!python3 %
```

**场景 2：文件操作**

```vim
" 列出当前目录
:!ls -la

" 创建目录
:!mkdir -p new_folder

" 复制文件
:!cp file.txt backup.txt

" 查看文件信息
:!file %
```

**场景 3：读取命令输出到缓冲区**

```vim
" 插入当前日期
:r !date

" 插入目录列表
:r !ls -la

" 插入文件内容
:r !cat other_file.txt

" 插入命令帮助
:r !gcc --help
```

**场景 4：通过命令过滤文本**

```vim
" 使用外部程序格式化代码
:%!clang-format

" 使用 sort 排序选中的行
:'<,'>!sort

" 使用 uniq 去重
:%!uniq

" 使用 grep 过滤
:%!grep pattern
```

**场景 5：写入到命令**

```vim
" 将缓冲区内容作为命令输入
:w !python3

" 通过邮件发送
:w !mail -s "Subject" user@example.com

" 复制到系统剪贴板（使用 xclip）
:w !xclip -selection clipboard
```

### 特殊符号

| 符号 | 含义 |
|------|------|
| `%` | 当前文件名 |
| `#` | 备用文件名 |
| `<cword>` | 当前单词 |
| `<cWORD>` | 当前大单词 |
| `<cfile>` | 光标下的文件名 |
| `%.ext` | 修改扩展名 |

### 示例

```vim
" 编译当前 C 文件
:!gcc % -o %.out

" 运行编译后的程序
:!./%.out

" 对当前单词执行 grep
:!grep -r <cword> .

" 在浏览器中打开当前文件
:!firefox %
```

---

## 总结

本文档介绍了 Aivim 尚未实现的 P0/P1 核心功能：

| 功能 | 优先级 | 主要用途 |
|------|--------|----------|
| 宏录制 | P1 | 自动化重复操作 |
| 撤销树 | P1 | 分支式撤销历史 |
| 高级搜索替换 | P2 | 批量文本处理 |
| 标记系统 | P2 | 快速位置跳转 |
| 跳转列表 | P2 | 导航历史回溯 |
| 行号显示 | P1 | 行号参考 |
| 配置系统 | P1 | 自定义编辑器行为 |
| 键位映射 | P1 | 自定义快捷键 |
| 外部命令 | P1 | 与系统交互 |

这些功能将大大提升编辑效率和用户体验，是 Aivim 向完整 Vim 体验迈进的重要组成部分。
