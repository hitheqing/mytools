--auto generated--
local SocialIsland_example_Client_Handle = {	}

---test_only_req  c2d test_only_req
function SocialIsland_example_Client_Handle.send_test_only_req()
	print(bWriteLog and string.format("SocialIsland_example_Client_Handle.send_test_only_req "))
	local res_param = {
	}
	local ds_net = require("ds_net")
	ds_net.SendMessage("SocialIsland.test_only_req", res_param)
end

---test_1_arg_req  c2d test_1_arg_req
---@param pa int32  1个参数
function SocialIsland_example_Client_Handle.send_test_1_arg_req(pa)
	print(bWriteLog and string.format("SocialIsland_example_Client_Handle.send_test_1_arg_req pa:%s",pa))
	local res_param = {
		pa = pa,
	}
	local ds_net = require("ds_net")
	ds_net.SendMessage("SocialIsland.test_1_arg_req", res_param)
end

---test_1_arg_rsp  c2d test_1_arg_rsp 2个参数返回
---@param pa int32 决斗类型（武器）
---@param pb int32 决斗类型（武器）
function SocialIsland_example_Client_Handle.on_test_1_arg_rsp(playerUid, message)
	print(bWriteLog and string.format("SocialIsland_example_Client_Handle.on_test_1_arg_rsp pa:%s, pb:%s",message.pa, message.pb))
end

---test_notify d2c 参数中带有注释和没注释
---@param nn1 int32  zhushi
---@param nocomment int32 
function SocialIsland_example_Client_Handle.on_test_notify(playerUid, message)
	print(bWriteLog and string.format("SocialIsland_example_Client_Handle.on_test_notify nn1:%s, nocomment:%s",message.nn1, message.nocomment))
end

return SocialIsland_example_Client_Handle
