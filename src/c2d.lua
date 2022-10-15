--auto generated--
local c2d = {	}



---pso_ready_req c2d 通知ds ready
function c2d.pso_ready_req()
	print(bWriteLog and string.format("c2d.pso_ready_req "))
end

---pso_next_point_req c2d 请求传送到下一个点位
function c2d.pso_next_point_req()
	print(bWriteLog and string.format("c2d.pso_next_point_req "))
end

---pso_early_exit_req c2d 提前退出
function c2d.pso_early_exit_req()
	print(bWriteLog and string.format("c2d.pso_early_exit_req "))
end

---pso_kick_action_req c2d 踢球动作发起时调用，用于ds计时用
---@param point_id int32 当前点位id，1~50
---@param point_index int32 当前点位index 1~10
function c2d.pso_kick_action_req(point_id, point_index)
	print(bWriteLog and string.format("c2d.pso_kick_action_req point_id:%s, point_index:%s", point_id, point_index))
end

---pso_cur_info_req c2d 断线重连请求当前ui数据
function c2d.pso_cur_info_req()
	print(bWriteLog and string.format("c2d.pso_cur_info_req "))
end

---pso_overlap_result_req c2d 上报踢球碰撞数据
---@param score int32 本次分数
function c2d.pso_overlap_result_req(score)
	print(bWriteLog and string.format("c2d.pso_overlap_result_req score:%s", score))
end

-----autogen update below-----

---pso_enter_req c2d 请求入场/再来一次
function c2d.pso_enter_req()
	print(bWriteLog and string.format("c2d.pso_enter_req "))
end

return c2d
