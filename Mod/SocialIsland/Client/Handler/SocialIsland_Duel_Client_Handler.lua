local SocialIsland_Duel_Client_Handler = {	}
local ds_net = require("ds_net")

---发起端收到反馈
---@param orign_uid number 
---@param duel_target_uid number 目标uid
---@param duel_response number 决斗反馈
function SocialIsland_Duel_Client_Handler.on_social_island_duel_start_rsp(message)
	print(bWriteLog and string.format("SocialIsland_Duel_Client_Handler.on_social_island_duel_start_rsp orign_uid:%s, duel_target_uid:%s, duel_response:%s",message.orign_uid, message.duel_target_uid, message.duel_response))
end

---被邀请方收到决斗申请
---@param orign_uid number 发起方uid
---@param duel_type number 决斗类型（武器）
function SocialIsland_Duel_Client_Handler.on_social_island_duel_apply_notify(message)
	print(bWriteLog and string.format("SocialIsland_Duel_Client_Handler.on_social_island_duel_apply_notify orign_uid:%s, duel_type:%s",message.orign_uid, message.duel_type))
end

---通知决斗结算
---@param duel_target_uid number 
---@param duel_type number 决斗类型（武器）
---@param winner_uid number 决斗反馈
function SocialIsland_Duel_Client_Handler.on_social_island_duel_result_notify(message)
	print(bWriteLog and string.format("SocialIsland_Duel_Client_Handler.on_social_island_duel_result_notify duel_target_uid:%s, duel_type:%s, winner_uid:%s",message.duel_target_uid, message.duel_type, message.winner_uid))
end

---请求发起决斗
---@param orign_uid number uid
---@param duel_target_uid number 目标uid
---@param duel_type number 决斗类型（武器）
function SocialIsland_Duel_Client_Handler.send_social_island_duel_start_req(orign_uid, duel_target_uid, duel_type)
	print(bWriteLog and string.format("SocialIsland_Duel_Client_Handler.send_social_island_duel_start_req orign_uid:%s, duel_target_uid:%s, duel_type:%s",orign_uid, duel_target_uid, duel_type))
	local res_param = {
		orign_uid = orign_uid,
		duel_target_uid = duel_target_uid,
		duel_type = duel_type,
	}
	ds_net.SendMessage("SocialIsland.social_island_duel_start_req", res_param)
end

---被邀请方对申请进行反馈 client->ds
---@param orign_uid number 发起方uid
---@param duel_target_uid number 
---@param duel_type number 决斗类型（武器）
---@param duel_response number 决斗反馈
function SocialIsland_Duel_Client_Handler.send_social_island_response_duel_req(orign_uid, duel_target_uid, duel_type, duel_response)
	print(bWriteLog and string.format("SocialIsland_Duel_Client_Handler.send_social_island_response_duel_req orign_uid:%s, duel_target_uid:%s, duel_type:%s, duel_response:%s",orign_uid, duel_target_uid, duel_type, duel_response))
	local res_param = {
		orign_uid = orign_uid,
		duel_target_uid = duel_target_uid,
		duel_type = duel_type,
		duel_response = duel_response,
	}
	ds_net.SendMessage("SocialIsland.social_island_response_duel_req", res_param)
end

return SocialIsland_Duel_Client_Handler
