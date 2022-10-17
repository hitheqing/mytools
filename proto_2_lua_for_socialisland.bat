@echo off
rem 关闭回显，避免输出重复命令

rem 设置变量延迟：在读取了一条完整的语句之后，不立即对该行的变量赋值
setlocal EnableDelayedExpansion

rem 设置bat当前文件所在目录为工作目录
pushd %~dp0

rem pb生成
rem LuaNet\LuaNetCreater.exe proto_to_pb %cd%\..\proto

rem DS协议代码生成
mytools.exe %cd%\..\proto\ds_client\SocialIsland %cd%\..\GameLua\Mod\ show_func_write

echo Run Success.
pause


