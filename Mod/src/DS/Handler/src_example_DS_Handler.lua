--auto generated--
local src_example_DS_Handler = {	}
local ds_net = require("ds_net")

---@class test_as_struct
---@field st number  啊啊啊
---@field st2 number  不不不
--local test_as_struct = {
--	st = 0,
--	st2 = 0,
--}

--- c2d test_only_req
function src_example_DS_Handler.on_test_only_req(playerUid, message)
	print(bWriteLog and string.format("src_example_DS_Handler.test_only_req playerUid:%s",playerUid))
end

--- c2d test_1_arg_req
---@param pa number  1个参数
function src_example_DS_Handler.on_test_1_arg_req(playerUid, message)
	print(bWriteLog and string.format("src_example_DS_Handler.on_test_1_arg_req playerUid:%s, pa:%s", playerUid, message.pa))
	local pa = 0
	local pb = {}
	src_example_DS_Handler.send_test_1_arg_rsp(playerUid, pa, pb)
end

--- c2d test_1_arg_rsp 2个参数返回
---@param pa number 决斗类型（武器）
---@param pb int32[] 决斗类型（武器）
function src_example_DS_Handler.send_test_1_arg_rsp(playerUid, pa, pb)
	print(bWriteLog and string.format("src_example_DS_Handler.send_test_1_arg_rsp playerUid:%s, pa:%s, pb:%s", playerUid, pa, pb))
	local res_param = {
		pa = pa,
		pb = pb,
	}

	ds_net.SendMessage("src.test_1_arg_rsp", res_param, playerUid)
end

---d2c 参数中带有注释和没注释
---@param nn1 number  zhushi
---@param nocomment number 
function src_example_DS_Handler.send_test_notify(playerUid, nn1, nocomment)
	print(bWriteLog and string.format("src_example_DS_Handler.send_test_notify playerUid:%s, nn1:%s, nocomment:%s", playerUid, nn1, nocomment))
	local res_param = {
		nn1 = nn1,
		nocomment = nocomment,
	}

	ds_net.SendMessage("src.test_notify", res_param, playerUid)
end

return src_example_DS_Handler
