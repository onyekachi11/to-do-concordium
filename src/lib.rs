use concordium_std::*;

//initialize a contract with user array
#[derive(Debug, PartialEq, Eq, SchemaType, Serial, Deserial, Clone)]
pub struct User {
    all_task: Vec<MainTask>,
}

//each user is able to create an array of tasks e.g [substack{}, substack{}]
#[derive(Debug, PartialEq, Eq, SchemaType, Serial, Deserial, Clone)]
pub struct MainTask {
    id: Address,
    description: String,
    tasks: Vec<SubTask>,
}

// Define the SubTask struct
#[derive(Debug, PartialEq, Eq, SchemaType, Serial, Deserial, Clone)]
pub struct SubTask {
    id: u32,
    description: String,
    completed: bool,
}

#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]

struct ContractState<S: HasStateApi = StateApi> {
    users: StateMap<Address, User, S>,
}

const MIN_AMOUNT: Amount = Amount::from_ccd(1);

//********************************* INITIATE CONTRACT************************/
#[init(contract = "TodoApp2")]
fn todo_app_init(
    _ctx: &InitContext,
    state_builder: &mut StateBuilder,
) -> InitResult<ContractState> {
    // Initialize an empty state for the contract
    let initial_state = ContractState {
        users: state_builder.new_map(),
    };

    Ok(initial_state)
}

//********************************* CREATE A MAIN TASK DESCRIPTION ************************/
#[derive(Serialize, SchemaType)]
struct TaskOption {
    description: String,
}

#[receive(
    contract = "TodoApp2",
    name = "create_task",
    payable,
    mutable,
    parameter = "TaskOption"
)]
fn create_task(
    ctx: &ReceiveContext,
    host: &mut Host<ContractState>,
    amount: Amount,
) -> ReceiveResult<()> {
    let parameter: TaskOption = ctx.parameter_cursor().get()?;

    ensure!(amount > MIN_AMOUNT);
    let new_task = MainTask {
        description: parameter.description,
        id: ctx.sender(),
        tasks: Vec::new(),
    };

    let user_address = ctx.sender();
    let mut user = host.state_mut().users.entry(user_address).or_insert(User {
        all_task: Vec::new(),
    });

    user.all_task.push(new_task);

    Ok(())
}

//********************************* CREATE A SUB TASK  ************************/
#[derive(Serial, Deserial, SchemaType)]
struct SubTaskOption {
    description: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReceiveError {
    User(String),
}

#[receive(
    contract = "TodoApp2",
    name = "create_sub_task",
    mutable,
    parameter = "SubTaskOption"
)]
fn create_sub_task(ctx: &ReceiveContext, host: &mut Host<ContractState>) -> ReceiveResult<()> {
    // Deserialize the parameter from the context
    let parameter: SubTaskOption = ctx.parameter_cursor().get()?;

    // Retrieve the user's to-do list
    let (user_result, _address) = get_user_todo(ctx, host)?;

    let user_address = ctx.sender();

    // Check if the user exists
    if let Some(mut user) = user_result {
        // Assuming we want to add the sub-task to the last main task
        if let Some(main_task) = user.all_task.last_mut() {
            // Generate a new sub-task ID based on the number of existing sub-tasks
            let sub_task_id = main_task.tasks.len() as u32;

            // Create and add the new sub-task to the main task's list of sub-tasks
            main_task.tasks.push(SubTask {
                id: sub_task_id,
                description: parameter.description,
                completed: false,
            });

            let _ = host.state_mut().users.insert(user_address, user);
        } else {
            // Handle case where there are no main tasks
            // return Err(Reject::default("No main tasks available to add a sub-task"));
        }
    } else {
        println!("No user found");
        // Optionally return an error here if user not found
        // return Err(Reject::User("User not found".into()));
    }

    Ok(())
}

//********************************* VIEW ALL USERS************************/
#[derive(Serial, Deserial, Clone, SchemaType)]
struct UserSummary {
    address: Address,
    task_count: Vec<MainTask>,
}

#[derive(Serial, Deserial, Clone, SchemaType)]
struct UserSummary2 {
    address: Address,
    user: User,
}
#[receive(
    contract = "TodoApp2",
    name = "view",
    mutable,
    return_value = "(Address, User)"
)]
fn view_todo(
    _ctx: &ReceiveContext,
    host: &mut Host<ContractState>,
) -> ReceiveResult<Vec<(Address, User)>> {
    let summaries: Vec<(Address, User)> = host
        .state()
        .users
        .iter()
        .map(|(address, user)| (*address, user.clone()))
        .collect();
    Ok(summaries)
}
// ReceiveResult<Option<StateMapIter<Address, User>>

#[derive(Serial, Deserial, SchemaType)]
struct UserAddress {
    address: Address,
}

//  ********************************* GET A USERS TODO LIST************************/
#[receive(
    contract = "TodoApp2",
    name = "get_user_todo",
    mutable,
    return_value = "(Option<User>, Address)"
)]
fn get_user_todo(
    ctx: &ReceiveContext,
    host: &mut Host<ContractState>,
) -> ReceiveResult<(Option<User>, Address)> {
    let user_address = ctx.sender();

    // Attempt to retrieve the user's data from the contract state
    let user_result = match host.state().users.get(&user_address) {
        Some(user_ref) => {
            let user: User = (*user_ref).clone();
            (Some(user), user_address)
        }
        None => (None, user_address),
    };

    Ok(user_result)
}

//********************************* MARK A SUBSTACK AS COMPLETED ************************/
#[derive(Serial, Deserial, SchemaType)]
struct Index {
    task_index: u64,
    sub_task_index: u64,
}
#[receive(
    contract = "TodoApp2",
    name = "mark_as_completed",
    mutable,
    parameter = "Index"
)]
fn mark_as_completed(
    ctx: &ReceiveContext,
    host: &mut Host<ContractState>,
    // task_index: usize,
) -> ReceiveResult<()> {
    let parameter: Index = ctx.parameter_cursor().get()?;
    let task_index = parameter.task_index as usize;
    // Get user todo
    let (user_result, _address) = get_user_todo(ctx, host)?;

    if let Some(mut user) = user_result {
        // Assuming index 0 for the first MainTask, adjust the index as needed
        if let Some(main_task) = user.all_task.get_mut(task_index) {
            let parameter: Index = ctx.parameter_cursor().get()?;
            let sub_task_index = parameter.sub_task_index as usize;

            main_task.tasks[sub_task_index as usize].completed = true
        } else {
            println!("No user found1");
        }
    } else {
        println!("No user found2");
    }

    Ok(())
}
