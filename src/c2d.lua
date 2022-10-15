--auto generated--
local c2d = {	}

---pso_cur_info_req c2d 断线重连请求当前ui数据
function c2d.pso_cur_info_req()
	print(bWriteLog and string.format("c2d.pso_cur_info_req "))
	local res_param = {
	}
	local ds_net    = require("ds_net")
	ds_net.SendMessage("SocialIsland.pso_cur_info_req", res_param, playerUid)
end

---pso_overlap_result_req c2d 上报踢球碰撞数据
---@param score int32 本次分数
function c2d.pso_overlap_result_req(score)
	print(bWriteLog and string.format("c2d.pso_overlap_result_req score:%s", score))
	local res_param = {
		score = score,
	}
	local ds_net    = require("ds_net")
	ds_net.SendMessage("SocialIsland.pso_overlap_result_req", res_param, playerUid)
end

return c2d
