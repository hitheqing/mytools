local SocialIsland_example_Ds_Handler = {	}
local ds_net = require("ds_net")

---@class test_as_struct  test_as_struct 结构定义
local test_as_struct = {
	---int32  啊啊啊	st = 0, 
	---int32  不不不	st2 = 0, 
}

--- c2d test_only_req
function SocialIsland_example_Ds_Handler.on_test_only_req(playerUid, message)
	print(bWriteLog and string.format("SocialIsland_example_Ds_Handler.test_only_req "))
end

--- c2d test_1_arg_req
---@param pa int32  1个参数
function SocialIsland_example_Ds_Handler.on_test_1_arg_req(playerUid, message)
	print(bWriteLog and string.format("SocialIsland_example_Ds_Handler.on_test_1_arg_req pa:%s",message.pa))
	local pa = 0
	local pb = {}
	SocialIsland_example_Ds_Handler.send_test_1_arg_rsp(playerUid, pa, pb)
end

--- c2d test_1_arg_rsp 2个参数返回
---@param pa int32 决斗类型（武器）
---@param pb int32[] 决斗类型（武器）
function SocialIsland_example_Ds_Handler.send_test_1_arg_rsp(playerUid, pa, pb)
	print(bWriteLog and string.format("SocialIsland_example_Ds_Handler.send_test_1_arg_rsp pa:%s, pb:%s",pa, pb))
	local res_param = {
		pa = pa,
		pb = pb,
	}
	ds_net.SendMessage("SocialIsland.test_1_arg_rsp", res_param, playerUid)
end

---d2c 参数中带有注释和没注释
---@param nn1 int32  zhushi
---@param nocomment int32 
function SocialIsland_example_Ds_Handler.send_test_notify(playerUid, nn1, nocomment)
	print(bWriteLog and string.format("SocialIsland_example_Ds_Handler.send_test_notify nn1:%s, nocomment:%s",nn1, nocomment))
	local res_param = {
		nn1 = nn1,
		nocomment = nocomment,
	}
	ds_net.SendMessage("SocialIsland.test_notify", res_param, playerUid)
end

return SocialIsland_example_Ds_Handler
