--auto generated--
local d2c = {    }

---pso_settle_notify d2c 结算
---@param total_score int32 总分
---@param score_list int32 每次的得分
---@param finish_type int32 0 正常完成，1 退出无奖励
---@param award_score int32 累计积分奖励
---@param max_score int32 赛季最大累计积分
---@param season_id int32 赛季id
function d2c.pso_settle_notify(total_score, score_list, finish_type, award_score, max_score, season_id)
	print(bWriteLog and string.format("d2c.pso_settle_notify total_score:%s, score_list:%s, finish_type:%s, award_score:%s, max_score:%s, season_id:%s", total_score, score_list, finish_type, award_score, max_score, season_id))
end

---pso_next_point_notify d2c ds通知传送到下一个点位
---@param point_id int32 当前点位id，1~50
---@param point_index int32 当前点位index 1~10
function d2c.pso_next_point_notify(point_id, point_index)
	print(bWriteLog and string.format("d2c.pso_next_point_notify point_id:%s, point_index:%s", point_id, point_index))
end

---pso_kick_score_notify d2c ds通知本次踢球的成绩
---@param point_id int32 当前点位id，1~50
---@param point_index int32 当前点位index 1~10
---@param score int32 本次得分
---@param total_score int32 当前总分
function d2c.pso_kick_score_notify(point_id, point_index, score, total_score)
	print(bWriteLog and string.format("d2c.pso_kick_score_notify point_id:%s, point_index:%s, score:%s, total_score:%s", point_id, point_index, score, total_score))
end

---pso_op_err_notify d2c 操作可能触发的错误码
---@param code int32 错误码
function d2c.pso_op_err_notify(code)
	print(bWriteLog and string.format("d2c.pso_op_err_notify code:%s", code))
end

---pso_cur_info_rsp c2d 断线重连请求当前ui数据
---@param point_id int32 当前点位id，1~50
---@param point_index int32 当前点位index 1~10
---@param total_score int32 当前总分
function d2c.pso_cur_info_rsp(point_id, point_index, total_score)
	print(bWriteLog and string.format("d2c.pso_cur_info_rsp point_id:%s, point_index:%s, total_score:%s", point_id, point_index, total_score))
end

return d2c
