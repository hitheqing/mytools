--auto generated--
local d2c = {    }

---pso_cur_info_rsp c2d 断线重连请求当前ui数据
---@param point_id int32 当前点位id，1~50
---@param point_index int32 当前点位index 1~10
---@param total_score int32 当前总分
function d2c.pso_cur_info_rsp(point_id, point_index, total_score)
	print(bWriteLog and string.format("d2c.pso_cur_info_rsp point_id:%s, point_index:%s, total_score:%s", point_id, point_index, total_score))
	local res_param = {
		point_id    = point_id,
		point_index = point_index,
		total_score = total_score,
	}
	local ds_net    = require("ds_net")
	ds_net.SendMessage("SocialIsland.pso_cur_info_rsp", res_param, playerUid)
end

return d2c
