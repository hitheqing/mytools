--auto generated--
local src_example_Client_Handler = {	}
local ds_net = require("ds_net")

---@class test_as_struct
---@field st number  啊啊啊
---@field st2 number  不不不
--local test_as_struct = {
--	st = 0,
--	st2 = 0,
--}

--- c2d test_only_req
function src_example_Client_Handler.send_test_only_req()
	print(bWriteLog and string.format("src_example_Client_Handler.test_only_req "))
	local res_param = {
	}

	ds_net.SendMessage("src.test_only_req", res_param)
end

--- c2d test_1_arg_req
---@param pa number  1个参数
function src_example_Client_Handler.send_test_1_arg_req(pa)
	print(bWriteLog and string.format("src_example_Client_Handler.send_test_1_arg_req pa:%s", pa))
	local res_param = {
		pa = pa,
	}

	ds_net.SendMessage("src.test_1_arg_req", res_param)
end

--- c2d test_1_arg_rsp 2个参数返回
---@param pa number 决斗类型（武器）
---@param pb int32[] 决斗类型（武器）
function src_example_Client_Handler.on_test_1_arg_rsp(message)
	print(bWriteLog and string.format("src_example_Client_Handler.on_test_1_arg_rsp pa:%s, pb:%s", message.pa, message.pb))
end

---d2c 参数中带有注释和没注释
---@param nn1 number  zhushi
---@param nocomment number 
function src_example_Client_Handler.on_test_notify(message)
	print(bWriteLog and string.format("src_example_Client_Handler.on_test_notify nn1:%s, nocomment:%s", message.nn1, message.nocomment))
end

return src_example_Client_Handler
