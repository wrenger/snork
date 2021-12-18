import subprocess
import json
import argparse
import numpy as np

from ConfigSpace import ConfigurationSpace
from ConfigSpace.hyperparameters import (CategoricalHyperparameter,
                                         UniformFloatHyperparameter,
                                         UniformIntegerHyperparameter,
                                         NormalFloatHyperparameter)
from ConfigSpace.conditions import InCondition
from dehb import DEHB
from smac.facade.smac_hpo_facade import SMAC4HPO
from smac.scenario.scenario import Scenario
from fanova import fANOVA

RANDOMSTATES = [42, 5, 725]
# To remove naming conflicts between agents, must be 2 characters long
PREFIXES = {"Tree": "t_", "Flood": "f_", "Mobility": "m_"}


def snake_from_config_wrapper(type, num_opponents, num_games_per_eval, timeout):
    def snake_from_config(cfg, *args, **kwargs):
        """ Run a rust snake with given configuration for NUM_GAMES_PER_EVAL games.
        Resulting score is percentage of games won.

        Parameters:
        ----------------
        cfg: Configuration (ConfigSpace.ConfigurationSpace.Configuration)
            Contains indexable parameters that define a snake.
        budget: timeout passed to snakes, affecting their cutoff point
        num_opponents: number of opponents the snake plays against during evaluation [1, 3]

        Returns:
        ----------------
        Percentage of won games. Crashing configurations automatically evaluate to the worst score.
        """
        # Turn Configuration into dict and remove None-values, remove prefixes
        cfg = {cfg['agent']: {k[2:]: cfg[k]
                              for k in cfg if cfg[k] and k != 'agent'}}

        # assemble cmd line call for the snake to be evaluated
        # if budget is None:
        #     budget = kwargs["max_budget"]
        fitness = 0.0
        time_used = 0.0
        opponents = ["'{\"Flood\":{}}'"] * num_opponents
        for rs in RANDOMSTATES:
            call = ["cargo run --release --bin simulate --"
                    f" '{json.dumps(cfg)}' " + " ".join(opponents) +
                    f" --game-count {num_games_per_eval}"
                    f" --timeout {timeout}"
                    f" --seed {rs}"
                    ]
            run = subprocess.run(call, capture_output=True,
                                shell=True, text=True, check=True)

            # extract games won and time taken from output
            games_won = run.stdout[9:run.stdout.find(",")]
            # inverse to minimize (treat games won as loss)
            fitness += 1.0 - int(games_won) / num_games_per_eval

            stderr = run.stderr
            time_used += float(stderr[stderr.rfind(f" "):stderr.rfind("ms")])

        fitness /= len(RANDOMSTATES)
        time_used / len(RANDOMSTATES)
        if type == "DEHB":
            return {"fitness": fitness,
                    "cost": time_used,
                    "info": {}}

        return fitness
    return snake_from_config


def get_cs(agentlist=["Flood"]):
    """Return full ConfigurationSpace for given list of agents."""

    cs = ConfigurationSpace()

    agent = CategoricalHyperparameter(
        "agent",
        agentlist,
        default_value="Flood" if "Flood" in agentlist else agentlist[0]
    )
    cs.add_hyperparameter(agent)

    if "Tree" in agentlist:
        tree_hps = {
            "mobility": 0.7,
            "mobility_decay": 0.0,
            "health": 0.012,
            "health_decay": 0.0,
            "len_advantage": 1.0,
            "len_advantage_decay": 0.0,
            "food_ownership": 0.65,
            "food_ownership_decay": 0.0,
            "centrality": 0.1,
            "centrality_decay": 0.0,
        }

        for hp in tree_hps:
            tree_hps[hp] = UniformFloatHyperparameter(
                PREFIXES["Tree"] + hp, lower=0, upper=1, default_value=tree_hps[hp])
            cs.add_hyperparameter(tree_hps[hp])
            cs.add_condition(InCondition(tree_hps[hp], agent, ["Tree"]))

    if "Mobility" in agentlist:
        mobility_hps = {
            "health_threshold": UniformIntegerHyperparameter(PREFIXES["Mobility"] + "health_threshold",
                                                             lower=0, upper=100, default_value=35),
            "min_len": UniformIntegerHyperparameter(PREFIXES["Mobility"] + "min_len",
                                                    lower=0, upper=121, default_value=8),
            "first_move_cost": UniformFloatHyperparameter(PREFIXES["Mobility"] + "first_move_cost",
                                                          lower=0, upper=3, default_value=1.0),
        }

        for hp in mobility_hps:
            cs.add_hyperparameter(mobility_hps[hp])
            cs.add_condition(InCondition(
                mobility_hps[hp], agent, ["Mobility"]))

    if "Flood" in agentlist:
        flood_hps = {
            "board_control": {"min": -6, "max": 6, "default": 0.9, "log": False},
            "board_control_decay": {"min": 1e-10, "max": 0.1, "default": 1e-10, "log": True},
            "board_control_offset": {"min": -5, "max": 5, "default": 0, "log": False},
            "health": {"min": -1.5, "max": 1.5, "default": 0.045, "log": False},
            "health_decay": {"min": 1e-10, "max": 0.1, "default": 1e-10, "log": True},
            "health_offset": {"min": -5, "max": 5, "default": 0, "log": False},
            "len_advantage": {"min": -15, "max": 15, "default": 6.4, "log": False},
            "len_advantage_decay": {"min": 1e-10, "max": 0.1, "default": 1e-10, "log": True},
            "len_advantage_offset": {"min": -5, "max": 5, "default": 0, "log": False},
            "food_distance": {"min": -2, "max": 2, "default": 0.415, "log": False},
            "food_distance_decay": {"min": 1e-10, "max": 0.1, "default": 1e-10, "log": True},
            "food_distance_offset": {"min": -5, "max": 5, "default": 0, "log": False},
        }

        for hp in flood_hps:
            flood_hps[hp] = UniformFloatHyperparameter(PREFIXES["Flood"] + hp,
                                                       lower=flood_hps[hp]["min"],
                                                       upper=flood_hps[hp]["max"],
                                                       default_value=flood_hps[hp]["default"],
                                                       log=flood_hps[hp]["log"])
            cs.add_hyperparameter(flood_hps[hp])
            cs.add_condition(InCondition(flood_hps[hp], agent, ["Flood"]))

    return cs


def optim_dehb(args):
    if args.runcount_limit == 0:
        args.runcount_limit = None

    f = snake_from_config_wrapper(
        "DEHB", args.num_opponents, args.num_games_per_eval, args.timeout)
    dehb = DEHB(f=f,
                cs=cs,
                dimensions=len(cs.get_hyperparameters()),
                min_budget=1,
                max_budget=2,
                n_workers=args.n_jobs,
                output_path=args.output_dir,
                save_smac=True)

    def_value = f(cs.get_default_configuration())
    print(
        f"Default Configuration evaluates to a win percentage of {(1 - def_value['fitness']) * 100:.2f}%")
    print(f"Starting Opimization of {args.runcount_limit} configurations...")

    trajectory, runtime, history = dehb.run(
        fevals=args.runcount_limit,
        total_cost=args.walltime,
        verbose=True,
        # parameters expected as **kwargs in target_function is passed here (surpassed by wrapper)
    )

    best_config = dehb.vector_to_configspace(dehb.inc_config)
    inc_value = f(best_config)
    print(
        f"Optimized Configuration {best_config} evaluates to a win percentage of {(1 - inc_value['fitness']) * 100:.2f}%")


def optim_smac(args):
    tae = snake_from_config_wrapper(
        "SMAC", args.num_opponents, args.num_games_per_eval, args.timeout)

    def_value = tae(cs.get_default_configuration(),
                    0,
                    num_opponents=args.num_opponents,
                    num_games_per_eval=args.num_games_per_eval,
                    timeout=args.timeout)
    print(
        f"Default Configuration evaluates to a win percentage of {(1 - def_value) * 100:.2f}%")
    print(
        f"Starting Opimization with walltime of {args.walltime/3600:.2f} hours...")

    scenario = Scenario({
        "run_obj": "quality",
        "runcount-limit": args.runcount_limit,
        "cs": cs,
        "deterministic": True,
        "wallclock_limit": args.walltime,
        "output_dir":args.output_dir,
        "shared_model": args.n_jobs > 1,
        "input_psmac_dirs": args.output_dir,
    })

    smac = SMAC4HPO(scenario=scenario,
                    rng=np.random.RandomState(RANDOMSTATES[0]),
                    tae_runner=tae)

    try:
        incumbent = smac.optimize()
    finally:
        incumbent = smac.solver.incumbent

    inc_value = tae(incumbent)
    print(
        f"Optimized Configuration {incumbent} evaluates to a win percentage of {(1 - inc_value) * 100:.2f}%")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Optimize hyperparameters of the rust snake.")
    parser.add_argument("--optimizer", type=str, choices=["DEHB", "SMAC"], default="DEHB",
                        help="Which optimizer is used for finding optimal hyperparameters.")
    parser.add_argument("--agents", choices=["Flood", "Tree", "Mobility"], nargs="+", default=["Flood"],
                        help="Which agents are included in the search space.")
    parser.add_argument("--num_games_per_eval", type=int, default=1000,
                        help="Number of games that each snake plays for evaluation")
    parser.add_argument("--timeout", type=int, default=0,
                        help="Timeout parameter passed directly to the snake. Larger values impact execution time a lot.")
    parser.add_argument("--runcount_limit", type=int, default=10000,
                        help="How many different configurations are evaluated at maximum. DEHB prioritizes this over runtime.")
    parser.add_argument("--walltime", type=int, default=10*60*60,
                        help="Max time in seconds optimizer runs for. DEHB overwrites this with runtime, pass 0 as runcount_limit to prevent this..")
    parser.add_argument("-j", "--n_jobs", type=int, default=8,
                        help="Number of workers that are started. Uses pSMAC if optimizer == SMAC.")
    parser.add_argument("-o", "--output_dir", type=str, default="./optim",
                        help="Location of log directory. pSMAC for example uses it also for internal communication,"
                             " so if using SMAC make sure that the directory is empty. '_output' will be appended.")
    parser.add_argument("--num_opponents", type=int, default=1,
                        help="Number of opponents agent is trained against: [1,3]")
    args = parser.parse_args()
    args.output_dir += "_output"
    cs = get_cs(args.agents)

    if args.optimizer == "DEHB":
        optim_dehb(args)
    elif args.optimizer == "SMAC":
        optim_smac(args)

    # evaluation and hyperparameter analysis -> PIMP
