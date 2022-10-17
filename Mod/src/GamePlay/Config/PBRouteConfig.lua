
--- 各个Mod PB协议路由定义配置文件，此文件乃自动生成，请勿手动修改
--- samizheng


--1.PB协议：客户端响应DS的协议表
local PBRouteConfig_Client =
{
	Duel =
	{
		moduleName = "GameLua.Mod.src.Client.Handler.src_Duel_Client_Handler",
		pbFileName = "src/Duel.pb",
		routes =
		{
			social_island_duel_start_req = true,
			social_island_duel_start_rsp = "on_social_island_duel_start_rsp",
			social_island_duel_apply_notify = "on_social_island_duel_apply_notify",
			social_island_response_duel_req = true,
			social_island_duel_result_notify = "on_social_island_duel_result_notify",
		},
	},
	example =
	{
		moduleName = "GameLua.Mod.src.Client.Handler.src_example_Client_Handler",
		pbFileName = "src/example.pb",
		routes =
		{
			test_only_req = true,
			test_1_arg_req = true,
			test_1_arg_rsp = "on_test_1_arg_rsp",
			test_notify = "on_test_notify",
		},
	},
}

--2.PB协议：DS响应客户端的协议表
local PBRouteConfig_DS =
{
	Duel =
	{
		moduleName = "GameLua.Mod.src.DS.Handler.src_Duel_DS_Handler",
		pbFileName = "src/Duel.pb",
		routes =
		{
			social_island_duel_start_req = "on_social_island_duel_start_req",
			social_island_duel_start_rsp = true,
			social_island_duel_apply_notify = true,
			social_island_response_duel_req = "on_social_island_response_duel_req",
			social_island_duel_result_notify = true,
		},
	},
	example =
	{
		moduleName = "GameLua.Mod.src.DS.Handler.src_example_DS_Handler",
		pbFileName = "src/example.pb",
		routes =
		{
			test_only_req = "on_test_only_req",
			test_1_arg_req = "on_test_1_arg_req",
			test_1_arg_rsp = true,
			test_notify = true,
		},
	},
}



if Client then
	return PBRouteConfig_Client
else
	return PBRouteConfig_DS
end
