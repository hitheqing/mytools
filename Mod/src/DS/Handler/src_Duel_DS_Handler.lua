--auto generated--
local src_Duel_DS_Handler = {	}
local ds_net = require("ds_net")

---请求发起决斗
---@param orign_uid number uid
---@param duel_target_uid number 目标uid
---@param duel_type number 决斗类型（武器）
function src_Duel_DS_Handler.on_social_island_duel_start_req(playerUid, message)
	print(bWriteLog and string.format("src_Duel_DS_Handler.on_social_island_duel_start_req playerUid:%s, orign_uid:%s, duel_target_uid:%s, duel_type:%s", playerUid, message.orign_uid, message.duel_target_uid, message.duel_type))
	local orign_uid = 0
	local duel_target_uid = 0
	local duel_response = 0
	src_Duel_DS_Handler.send_social_island_duel_start_rsp(playerUid, orign_uid, duel_target_uid, duel_response)
end

---发起端收到反馈
---@param orign_uid number 
---@param duel_target_uid number 目标uid
---@param duel_response number 决斗反馈
function src_Duel_DS_Handler.send_social_island_duel_start_rsp(playerUid, orign_uid, duel_target_uid, duel_response)
	print(bWriteLog and string.format("src_Duel_DS_Handler.send_social_island_duel_start_rsp playerUid:%s, orign_uid:%s, duel_target_uid:%s, duel_response:%s", playerUid, orign_uid, duel_target_uid, duel_response))
	local res_param = {
		orign_uid = orign_uid,
		duel_target_uid = duel_target_uid,
		duel_response = duel_response,
	}

	ds_net.SendMessage("src.social_island_duel_start_rsp", res_param, playerUid)
end

---被邀请方收到决斗申请
---@param orign_uid number 发起方uid
---@param duel_type number 决斗类型（武器）
function src_Duel_DS_Handler.send_social_island_duel_apply_notify(playerUid, orign_uid, duel_type)
	print(bWriteLog and string.format("src_Duel_DS_Handler.send_social_island_duel_apply_notify playerUid:%s, orign_uid:%s, duel_type:%s", playerUid, orign_uid, duel_type))
	local res_param = {
		orign_uid = orign_uid,
		duel_type = duel_type,
	}

	ds_net.SendMessage("src.social_island_duel_apply_notify", res_param, playerUid)
end

---被邀请方对申请进行反馈 client->ds
---@param orign_uid number 发起方uid
---@param duel_target_uid number 
---@param duel_type number 决斗类型（武器）
---@param duel_response number 决斗反馈
function src_Duel_DS_Handler.on_social_island_response_duel_req(playerUid, message)
	print(bWriteLog and string.format("src_Duel_DS_Handler.on_social_island_response_duel_req playerUid:%s, orign_uid:%s, duel_target_uid:%s, duel_type:%s, duel_response:%s", playerUid, message.orign_uid, message.duel_target_uid, message.duel_type, message.duel_response))
end

---通知决斗结算
---@param duel_target_uid number 
---@param duel_type number 决斗类型（武器）
---@param winner_uid number 决斗反馈
function src_Duel_DS_Handler.send_social_island_duel_result_notify(playerUid, duel_target_uid, duel_type, winner_uid)
	print(bWriteLog and string.format("src_Duel_DS_Handler.send_social_island_duel_result_notify playerUid:%s, duel_target_uid:%s, duel_type:%s, winner_uid:%s", playerUid, duel_target_uid, duel_type, winner_uid))
	local res_param = {
		duel_target_uid = duel_target_uid,
		duel_type = duel_type,
		winner_uid = winner_uid,
	}

	ds_net.SendMessage("src.social_island_duel_result_notify", res_param, playerUid)
end

return src_Duel_DS_Handler
